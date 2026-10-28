use super::StoredSession;
use std::fmt;

const CREDENTIAL_TARGET: &str = "TerrieFlow.Client.CabinetSession.v1";

pub(crate) trait SessionStore: Send + Sync {
    fn load(&self) -> Result<Option<StoredSession>, SessionStoreError>;
    fn save(&self, session: &StoredSession) -> Result<(), SessionStoreError>;
    fn clear(&self) -> Result<(), SessionStoreError>;
}

#[derive(Clone, Copy, Default)]
pub(crate) struct CredentialSessionStore;

#[derive(Debug)]
pub(crate) enum SessionStoreError {
    #[cfg(not(windows))]
    Unavailable,
    InvalidData,
    TooLarge,
    Platform(u32),
}

impl fmt::Display for SessionStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            #[cfg(not(windows))]
            Self::Unavailable => "secure credential storage is unavailable",
            Self::InvalidData => "stored account session is invalid",
            Self::TooLarge => "account session exceeds the secure storage limit",
            Self::Platform(code) => {
                return write!(formatter, "secure credential storage failed ({code})");
            }
        })
    }
}

impl std::error::Error for SessionStoreError {}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::ptr;
    use std::slice;
    use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, GetLastError};
    use windows_sys::Win32::Security::Credentials::{
        CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CredDeleteW, CredFree,
        CredReadW, CredWriteW,
    };
    use zeroize::Zeroize;

    const MAX_CREDENTIAL_BLOB_BYTES: usize = 2_560;

    impl SessionStore for CredentialSessionStore {
        fn load(&self) -> Result<Option<StoredSession>, SessionStoreError> {
            let target = wide(CREDENTIAL_TARGET);
            let mut raw = ptr::null_mut::<CREDENTIALW>();
            // SAFETY: target is a NUL-terminated UTF-16 buffer and raw is a valid out pointer.
            let success = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut raw) };
            if success == 0 {
                // SAFETY: GetLastError has no preconditions and is read immediately after failure.
                let code = unsafe { GetLastError() };
                return if code == ERROR_NOT_FOUND {
                    Ok(None)
                } else {
                    Err(SessionStoreError::Platform(code))
                };
            }
            if raw.is_null() {
                return Err(SessionStoreError::InvalidData);
            }

            // SAFETY: CredReadW returned a valid CREDENTIALW allocation until CredFree is called.
            let credential = unsafe { &*raw };
            let size = credential.CredentialBlobSize as usize;
            if size == 0 || size > MAX_CREDENTIAL_BLOB_BYTES || credential.CredentialBlob.is_null() {
                // SAFETY: raw was allocated by CredReadW.
                unsafe { CredFree(raw.cast()) };
                return Err(SessionStoreError::InvalidData);
            }
            // SAFETY: the credential owns CredentialBlobSize readable bytes.
            let mut bytes = unsafe {
                slice::from_raw_parts(credential.CredentialBlob.cast_const(), size).to_vec()
            };
            // SAFETY: raw was allocated by CredReadW and bytes now owns a copy.
            unsafe { CredFree(raw.cast()) };
            let decoded = serde_json::from_slice::<StoredSession>(&bytes)
                .map_err(|_| SessionStoreError::InvalidData);
            bytes.zeroize();
            decoded.map(Some)
        }

        fn save(&self, session: &StoredSession) -> Result<(), SessionStoreError> {
            let mut bytes = serde_json::to_vec(session).map_err(|_| SessionStoreError::InvalidData)?;
            if bytes.is_empty() || bytes.len() > MAX_CREDENTIAL_BLOB_BYTES {
                bytes.zeroize();
                return Err(SessionStoreError::TooLarge);
            }
            let mut target = wide(CREDENTIAL_TARGET);
            let mut username = wide("TerrieFlow");
            let credential = CREDENTIALW {
                Type: CRED_TYPE_GENERIC,
                TargetName: target.as_mut_ptr(),
                CredentialBlobSize: bytes.len() as u32,
                CredentialBlob: bytes.as_mut_ptr(),
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                UserName: username.as_mut_ptr(),
                ..CREDENTIALW::default()
            };
            // SAFETY: all pointers remain valid for the duration of CredWriteW, which copies them.
            let success = unsafe { CredWriteW(&credential, 0) };
            bytes.zeroize();
            if success == 0 {
                // SAFETY: GetLastError is read immediately after failure.
                Err(SessionStoreError::Platform(unsafe { GetLastError() }))
            } else {
                Ok(())
            }
        }

        fn clear(&self) -> Result<(), SessionStoreError> {
            let target = wide(CREDENTIAL_TARGET);
            // SAFETY: target is a NUL-terminated UTF-16 buffer.
            let success = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
            if success != 0 {
                return Ok(());
            }
            // SAFETY: GetLastError is read immediately after failure.
            let code = unsafe { GetLastError() };
            if code == ERROR_NOT_FOUND {
                Ok(())
            } else {
                Err(SessionStoreError::Platform(code))
            }
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
}

#[cfg(not(windows))]
impl SessionStore for CredentialSessionStore {
    fn load(&self) -> Result<Option<StoredSession>, SessionStoreError> {
        Err(SessionStoreError::Unavailable)
    }

    fn save(&self, _session: &StoredSession) -> Result<(), SessionStoreError> {
        Err(SessionStoreError::Unavailable)
    }

    fn clear(&self) -> Result<(), SessionStoreError> {
        Err(SessionStoreError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_target_is_stable_and_contains_no_user_data() {
        assert_eq!(CREDENTIAL_TARGET, "TerrieFlow.Client.CabinetSession.v1");
    }
}
