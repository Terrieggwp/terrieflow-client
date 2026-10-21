//! Windows shell identity for taskbar grouping and icon caching.

const APP_USER_MODEL_ID: &str = "TerrieFlow.Client";

pub(crate) fn set_process_app_id() {
    let app_id: Vec<u16> = APP_USER_MODEL_ID.encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr())
    };
    if result < 0 {
        eprintln!("Windows could not set the TerrieFlow application identity: 0x{result:08x}");
    }
}

#[cfg(test)]
mod tests {
    use super::APP_USER_MODEL_ID;

    #[test]
    fn application_identity_uses_the_terrieflow_brand() {
        assert_eq!(APP_USER_MODEL_ID, "TerrieFlow.Client");
    }
}
