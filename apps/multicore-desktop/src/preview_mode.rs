//! Safe, UI-only preview mode for the temporary TerrieFlow development build.
//!
//! A debug executable named `TerrieFlow-Preview.exe` uses its own data directory and never
//! starts the daemon, cores, updater, service installer, or Windows settings integration.

use std::path::PathBuf;

use slint::{ModelRc, SharedString, VecModel};

use crate::{AppWindow, UiCatalogGroup, UiCatalogNode, UiSubscriptionProfile};

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

/// Installs realistic, inert fixture data in the isolated executable. This gives design
/// reviews a representative home page without asking a daemon, subscription endpoint, or
/// Windows networking API for anything.
pub(crate) fn apply_demo(ui: &AppWindow) {
    ui.set_terrieflow_preview(true);
    ui.set_terrieflow_preview_grace(false);
    ui.set_navigation_collapsed(false);
    ui.set_has_profile(true);
    ui.set_status_eyebrow(text("Безопасный предпросмотр"));
    ui.set_headline(text("TerrieFlow Client"));
    ui.set_supporting(text("Сетевые функции отключены"));
    ui.set_status_tone(text("success"));
    ui.set_toggle_title(text("Защита включена"));
    ui.set_toggle_on(true);
    ui.set_toggle_busy(false);
    ui.set_toggle_detail_is_exit(true);
    ui.set_primary_label(text("Недоступно в предпросмотре"));
    ui.set_primary_enabled(false);
    ui.set_primary_queueable(false);
    ui.set_node_name(text("Finland"));
    ui.set_node_icon(text("🇫🇮"));
    ui.set_node_icon_image(crate::emoji_images::image_for("🇫🇮"));
    ui.set_node_latency(text("34 мс"));
    ui.set_exit_now_name(text("Finland"));
    ui.set_exit_now_icon(text("🇫🇮"));
    ui.set_exit_now_icon_image(crate::emoji_images::image_for("🇫🇮"));
    ui.set_session_uptime(text("00:12:43"));
    ui.set_traffic_down(text("48.6 МБ/с"));
    ui.set_traffic_up(text("6.2 МБ/с"));

    ui.set_subscription_title(text("Premium Pro"));
    ui.set_subscription_initial(text("P"));
    ui.set_subscription_usage(text("112.4 GB / 200 GB"));
    ui.set_subscription_expiry(text("Осталось 12 дней"));
    ui.set_subscription_remaining(text("87.6 GB"));
    ui.set_subscription_days(text("12 дн."));
    ui.set_subscription_used_fraction(0.562);
    ui.set_subscription_unlimited(false);
    ui.set_subscription_quota_tone(text("accent"));
    ui.set_subscription_quota_tooltip(text(
        "Трафик: 112.4 / 200 GB · осталось 87.6 GB · 12 дней",
    ));
    ui.set_subscription_updated(text("Сегодня, 14:27"));
    ui.set_subscription_refresh_available(false);
    ui.set_subscription_refresh_enabled(false);
    ui.set_subscription_management_enabled(false);
    ui.set_active_profile_id(text("premium-pro"));
    ui.set_subscription_profiles(ModelRc::new(VecModel::from(vec![preview_profile()])));

    ui.set_catalog_groups(ModelRc::new(VecModel::from(vec![UiCatalogGroup {
        id: text("auto"),
        icon: text("🌐"),
        icon_image: crate::emoji_images::image_for("🌐"),
        icon_loading: false,
        label: text("Автовыбор"),
        selected: true,
    }])));
    ui.set_catalog_nodes(ModelRc::new(VecModel::from(preview_nodes())));
    ui.set_catalog_selection_enabled(false);
    ui.set_catalog_loading(false);
    ui.set_catalog_error(text(""));
    ui.set_latency_check_available(false);
    ui.set_latency_check_loading(false);
    ui.set_system_name(text("Windows 11"));
    ui.set_reduced_motion(true);
    ui.set_service_card_visible(false);
    ui.set_auto_connect_on_startup(false);
    ui.set_tun_enabled(false);
}

fn preview_nodes() -> Vec<UiCatalogNode> {
    [
        ("finland", "🇫🇮", "Finland", true, "34 мс", "success"),
        ("sweden", "🇸🇪", "Sweden", false, "38 мс", "success"),
        (
            "netherlands",
            "🇳🇱",
            "Netherlands",
            false,
            "52 мс",
            "success",
        ),
        ("germany", "🇩🇪", "Germany", false, "68 мс", "warning"),
        (
            "united-states",
            "🇺🇸",
            "United States",
            false,
            "142 мс",
            "warning",
        ),
        ("albania", "🇦🇱", "Albania", false, "214 мс", "danger"),
    ]
    .into_iter()
    .map(|(id, icon, label, selected, latency, tone)| UiCatalogNode {
        id: text(id),
        icon: text(icon),
        icon_image: crate::emoji_images::image_for(icon),
        icon_loading: false,
        label: text(label),
        selected,
        latency_text: text(latency),
        latency_tone: text(tone),
        measuring: false,
    })
    .collect()
}

fn preview_profile() -> UiSubscriptionProfile {
    UiSubscriptionProfile {
        id: text("premium-pro"),
        title: text("Premium Pro"),
        usage: text("112.4 GB / 200 GB"),
        expiry: text("Осталось 12 дней"),
        active: true,
        available: true,
        has_logo: false,
        logo: Default::default(),
        logo_small: Default::default(),
        fallback: text("P"),
        remaining: text("87.6 GB"),
        days: text("12 дн."),
        used_fraction: 0.562,
        unlimited: false,
        quota_tone: text("accent"),
        expired: false,
        quota_tooltip: text("Трафик: 112.4 / 200 GB · осталось 87.6 GB · 12 дней"),
    }
}

fn text(value: &str) -> SharedString {
    value.into()
}
