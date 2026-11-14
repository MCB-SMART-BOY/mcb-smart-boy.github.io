#[cfg(feature = "hydrate")]
use crate::landing::LandingSection;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
fn focus_element(element: Option<web_sys::HtmlElement>) {
    if let Some(element) = element {
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        if let Err(source) = element.focus_with_options(&options) {
            web_sys::console::error_2(
                &"Failed to focus sidebar navigation destination".into(),
                &source,
            );
        }
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn focus_landing_section(section: LandingSection) {
    let target = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(section.id()))
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    focus_element(target);
}

#[cfg(feature = "hydrate")]
pub(crate) fn landing_section_from_href(href: &str) -> Option<LandingSection> {
    LandingSection::ALL
        .into_iter()
        .find(|section| section.href() == href)
}

#[cfg(feature = "hydrate")]
pub(crate) fn focus_main_after_sidebar_navigation() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let has_navigation_focus = document
        .active_element()
        .is_some_and(|element| is_main_navigation_link(&element));
    if !has_navigation_focus {
        return;
    }
    let main = document
        .get_element_by_id("main")
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    focus_element(main);
}

#[cfg(feature = "hydrate")]
pub(crate) fn is_main_navigation_link(element: &web_sys::Element) -> bool {
    const SELECTOR: &str =
        ".site-sidebar a[href]:not([data-sidebar-level]):not([data-reading-branch])";
    let link = match element.closest(SELECTOR) {
        Ok(Some(link)) => link,
        Ok(None) => return false,
        Err(source) => {
            web_sys::console::error_2(&"Failed to find content navigation link".into(), &source);
            return false;
        }
    };
    link.get_attribute("href")
        .is_some_and(|href| href.starts_with('/'))
}

#[cfg(feature = "hydrate")]
pub(crate) fn focus_element_for_shell(element: Option<web_sys::HtmlElement>) {
    focus_element(element);
}

pub(crate) fn is_unmodified_click(event: &leptos::ev::MouseEvent) -> bool {
    !event.default_prevented()
        && event.button() == 0
        && !event.ctrl_key()
        && !event.meta_key()
        && !event.shift_key()
        && !event.alt_key()
}
