use crate::{
    locale::Locale,
    preferences::{UiPreferences, find_cookie_value, parse_preferences},
};

#[cfg(feature = "hydrate")]
use crate::preferences::read_browser_cookies;
#[cfg(feature = "hydrate")]
use leptos::prelude::{GetUntracked, RwSignal, Set, batch};

#[cfg(any(feature = "hydrate", test))]
pub(super) fn parse_browser_preferences(cookies: Option<&str>) -> (UiPreferences, Locale) {
    let preferences =
        parse_preferences(cookies.and_then(|value| find_cookie_value(value, "mcb-ui")));
    let locale =
        crate::locale::parse_locale(cookies.and_then(|value| find_cookie_value(value, "mcb-lang")));
    (preferences, locale)
}

#[cfg(feature = "hydrate")]
pub(super) fn restore_browser_preferences(
    preferences: RwSignal<UiPreferences>,
    locale: RwSignal<Locale>,
    has_restored: RwSignal<bool>,
) {
    if has_restored.get_untracked() {
        return;
    }
    let cookies = read_browser_cookies();
    let (restored_preferences, restored_locale) = parse_browser_preferences(cookies.as_deref());
    batch(move || {
        preferences.set(restored_preferences);
        locale.set(restored_locale);
        has_restored.set(true);
    });
}

#[cfg(feature = "hydrate")]
fn set_document_attribute(
    element: &web_sys::Element,
    name: &str,
    value: &str,
    operation: &'static str,
) {
    if let Err(source) = element.set_attribute(name, value) {
        web_sys::console::error_2(&operation.into(), &source);
    }
}

#[cfg(feature = "hydrate")]
pub(super) fn sync_document_preferences(preferences: UiPreferences) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(root) = document.document_element() {
        set_document_attribute(
            &root,
            "data-theme",
            preferences.theme.as_str(),
            "Failed to update document theme",
        );
        set_document_attribute(
            &root,
            "data-ui",
            &preferences.cookie_value(),
            "Failed to update document UI state",
        );
    }
    match document.query_selector("meta[name=theme-color]") {
        Ok(Some(meta)) => set_document_attribute(
            &meta,
            "content",
            preferences.theme.color(),
            "Failed to update browser theme color",
        ),
        Ok(None) => {}
        Err(source) => web_sys::console::error_2(
            &"Failed to find browser theme color metadata".into(),
            &source,
        ),
    }
}

#[cfg(feature = "hydrate")]
pub(super) fn sync_document_locale(locale: Locale) {
    let root = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element());
    if let Some(root) = root {
        set_document_attribute(
            &root,
            "lang",
            locale.as_str(),
            "Failed to update document language",
        );
    }
}

#[cfg(feature = "hydrate")]
pub(super) fn mark_document_hydrated() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Some(root) = window
        .document()
        .and_then(|document| document.document_element())
    else {
        return false;
    };
    set_document_attribute(
        &root,
        "data-hydrated",
        "true",
        "Failed to mark document as hydrated",
    );
    true
}
