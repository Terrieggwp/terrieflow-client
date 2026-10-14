//! Safe, UI-only preview mode for the temporary TerrieFlow development build.
//!
//! A debug executable named `TerrieFlow-Preview.exe` uses its own data directory and never
//! starts the daemon, cores, updater, service installer, or Windows settings integration.

use std::path::PathBuf;

pub(crate) fn enabled() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    let requested = std::env::var_os("MULTICORE_PREVIEW_INSTANCE").as_deref()
        == Some(std::ffi::OsStr::new("1"));
    requested
        || std::env::current_exe()
            .ok()
            .and_then(|path| path.file_stem().map(|stem| stem.to_owned()))
            .and_then(|stem| stem.to_str().map(str::to_owned))
            .is_some_and(|stem| stem.eq_ignore_ascii_case("TerrieFlow-Preview"))
}

pub(crate) fn data_root() -> Option<PathBuf> {
    enabled().then(|| {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
            .unwrap_or_else(std::env::temp_dir)
            .join("preview-data")
            .join("MultiCore")
    })
}
