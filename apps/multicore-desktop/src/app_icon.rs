#[cfg(any(windows, test))]
pub(crate) const TRAY_ICON_SIZE: u32 = 32;

#[cfg(test)]
const APP_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app-icon-32.rgba"));
#[cfg(test)]
const APP_ICON_64_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app-icon-64.rgba"));
const TRAY_ACTIVE_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tray-active-32.rgba"));
const TRAY_ACTIVE_64_RGBA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/tray-active-64.rgba"));
const TRAY_INACTIVE_RGBA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/tray-inactive-32.rgba"));
const TRAY_INACTIVE_64_RGBA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/tray-inactive-64.rgba"));

#[cfg(test)]
pub(crate) fn rgba(size: u32) -> Vec<u8> {
    match size {
        32 => APP_ICON_RGBA.to_vec(),
        64 => APP_ICON_64_RGBA.to_vec(),
        _ => panic!("unsupported internal app icon size: {size}"),
    }
}

/// The connection state still carries the details used by the UI and tooltip. The tray renders
/// it as two TerrieFlow brand states: colour while connected, white in every non-connected state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TrayIconState {
    #[default]
    Disconnected,
    Connecting,
    /// Connected through the system proxy.
    Proxy,
    /// Connected through TUN (all traffic).
    Tun,
    Error,
}

impl TrayIconState {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 5] = [
        Self::Disconnected,
        Self::Connecting,
        Self::Proxy,
        Self::Tun,
        Self::Error,
    ];

    fn is_connected(self) -> bool {
        matches!(self, Self::Proxy | Self::Tun)
    }
}

/// The tray icon for `state`, `size` px square (32 or 64), as straight RGBA.
#[cfg(any(windows, target_os = "linux", test))]
pub(crate) fn tray_state_rgba(state: TrayIconState, size: u32) -> Vec<u8> {
    let pixels: &[u8] = match (state.is_connected(), size) {
        (true, 32) => TRAY_ACTIVE_RGBA,
        (true, 64) => TRAY_ACTIVE_64_RGBA,
        (false, 32) => TRAY_INACTIVE_RGBA,
        (false, 64) => TRAY_INACTIVE_64_RGBA,
        _ => panic!("unsupported internal tray icon size: {size}"),
    };
    pixels.to_vec()
}

#[cfg(test)]
mod tests {
    use super::{TRAY_ICON_SIZE, TrayIconState, rgba, tray_state_rgba};

    #[test]
    fn app_and_tray_icons_are_generated_from_the_terrieflow_sources() {
        let runtime_source = include_str!("app_icon.rs");
        let build_source = include_str!("../build.rs");

        assert!(runtime_source.contains("tray-active-32.rgba"));
        assert!(runtime_source.contains("tray-inactive-32.rgba"));
        assert!(build_source.contains("include_bytes!(\"assets/app-icon.svg\")"));
        assert!(build_source.contains("include_bytes!(\"assets/tray-active.svg\")"));
        assert!(build_source.contains("include_bytes!(\"assets/tray-inactive.svg\")"));
        assert!(build_source.contains("TRAY_ICON_SIZES: [u32; 2] = [32, 64]"));
    }

    #[test]
    fn app_icon_is_transparent_rgba_with_the_terrieflow_colours() {
        let pixels = rgba(TRAY_ICON_SIZE);
        assert_eq!(pixels.len(), (TRAY_ICON_SIZE * TRAY_ICON_SIZE * 4) as usize);

        let mut has_blue = false;
        let mut has_neutral = false;
        let mut has_transparency = false;
        for pixel in pixels.as_chunks::<4>().0 {
            has_blue |= pixel[2] > 180 && pixel[0] < 120 && pixel[3] > 200;
            has_neutral |= pixel[0] > 190 && pixel[1] > 190 && pixel[2] > 190 && pixel[3] > 200;
            has_transparency |= pixel[3] == 0;
        }

        assert!(has_blue);
        assert!(has_neutral);
        assert!(has_transparency);
        for (x, y) in [
            (0, 0),
            (TRAY_ICON_SIZE - 1, 0),
            (0, TRAY_ICON_SIZE - 1),
            (TRAY_ICON_SIZE - 1, TRAY_ICON_SIZE - 1),
        ] {
            let alpha = pixels[((y * TRAY_ICON_SIZE + x) * 4 + 3) as usize];
            assert_eq!(
                alpha, 0,
                "app icon corner ({x}, {y}) must stay transparent"
            );
        }
    }

    #[test]
    fn tray_uses_colour_only_while_connected() {
        for size in [32, 64] {
            let inactive = tray_state_rgba(TrayIconState::Disconnected, size);
            assert_eq!(inactive, tray_state_rgba(TrayIconState::Connecting, size));
            assert_eq!(inactive, tray_state_rgba(TrayIconState::Error, size));

            let active = tray_state_rgba(TrayIconState::Proxy, size);
            assert_eq!(active, tray_state_rgba(TrayIconState::Tun, size));
            assert_ne!(active, inactive);
            assert_eq!(active.len(), (size * size * 4) as usize);
            assert_eq!(inactive.len(), (size * size * 4) as usize);
        }
    }

    #[test]
    fn every_connection_state_has_a_supported_tray_icon() {
        for state in TrayIconState::ALL {
            assert_eq!(tray_state_rgba(state, 32).len(), 32 * 32 * 4);
            assert_eq!(tray_state_rgba(state, 64).len(), 64 * 64 * 4);
        }
    }
}
