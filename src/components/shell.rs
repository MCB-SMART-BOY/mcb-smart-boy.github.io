use leptos::{html, prelude::*};

use crate::{
    landing::{LandingSection, provide_landing_context},
    locale::Locale,
    preferences::{SIDEBAR_MOBILE_BREAKPOINT_PX, UiPreferences},
};
use leptos_router::hooks::use_location;

#[cfg(feature = "hydrate")]
use super::navigation_behavior::{
    focus_element_for_shell, focus_landing_section, focus_main_after_sidebar_navigation,
    is_main_navigation_link, landing_section_from_href,
};
use super::{
    article_outline::provide_article_outline_context,
    current_directory::{CurrentDirectory, has_article_directory},
    icons::{AsteriskGlyph, Icon, IconKind},
    reading_navigation::ReadingNavigation,
    sidebar::{Sidebar, SidebarBrand, SidebarNavigation},
    topbar::Topbar,
};

#[cfg(test)]
use super::browser_preferences::parse_browser_preferences;
#[cfg(feature = "hydrate")]
use super::browser_preferences::{
    mark_document_hydrated, restore_browser_preferences, sync_document_locale,
    sync_document_preferences,
};
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

// Keep the shell's SSR tree and drawer lifecycle under one reactive owner;
// this component intentionally exceeds the helper-function length limit.
#[component]
pub fn SiteShell(children: Children) -> impl IntoView {
    let preferences = use_context::<RwSignal<UiPreferences>>()
        .unwrap_or_else(|| RwSignal::new(UiPreferences::default()));
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let is_hydrated = RwSignal::new(false);
    #[cfg(feature = "hydrate")]
    let has_restored_browser_preferences = RwSignal::new(false);
    let viewport_width = RwSignal::new(1440_u16);
    let is_drawer_open = RwSignal::new(false);
    let focus_main_after_close = RwSignal::new(false);
    let landing_focus_after_close = RwSignal::new(None::<LandingSection>);
    let drawer = NodeRef::<html::Dialog>::new();
    let trigger = NodeRef::<html::Button>::new();
    let mobile_reading_request = RwSignal::new(0_u64);
    let workspace = NodeRef::<html::Div>::new();
    let pathname = use_location().pathname;
    provide_article_outline_context(pathname);
    provide_landing_context(pathname);
    let is_mobile_visible = Signal::derive(move || {
        viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX && is_drawer_open.get()
    });

    #[cfg(feature = "hydrate")]
    {
        super::directory_focus::watch_directory_focus(workspace, pathname);
        let location = use_location();
        let initial_path = location.pathname.get_untracked();
        Effect::new(move |previous_path: Option<String>| {
            let path = location.pathname.get();
            if previous_path.as_deref().unwrap_or(initial_path.as_str()) != path {
                focus_main_after_sidebar_navigation();
            }
            path
        });
        Effect::new(move || {
            restore_browser_preferences(preferences, locale, has_restored_browser_preferences);
        });
        Effect::new(move || {
            if has_restored_browser_preferences.get() {
                sync_document_preferences(preferences.get());
            }
        });
        Effect::new(move || {
            let is_collapsed = preferences.get().is_sidebar_collapsed;
            let is_mobile = viewport_width.get() < SIDEBAR_MOBILE_BREAKPOINT_PX;
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return;
            };
            let hidden_focus = if is_mobile {
                document.query_selector(".site-sidebar :focus")
            } else if is_collapsed {
                document.query_selector(".site-sidebar .reading-navigation :focus, .site-sidebar .sidebar-resizer:focus")
            } else {
                return;
            };
            if hidden_focus.ok().flatten().is_some() {
                let replacement = if is_mobile {
                    trigger.get_untracked().map(|element| element.into())
                } else {
                    document
                        .query_selector(".site-sidebar .brand-expand")
                        .ok()
                        .flatten()
                        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
                };
                focus_element_for_shell(replacement);
            }
        });
        Effect::new(move || {
            if has_restored_browser_preferences.get() {
                sync_document_locale(locale.get());
            }
        });
        Effect::new(move || {
            if !has_restored_browser_preferences.get() {
                return;
            }
            if let Some(window) = web_sys::window() {
                if let Ok(width) = window.inner_width() {
                    viewport_width.set(
                        width
                            .as_f64()
                            .unwrap_or(f64::from(SIDEBAR_MOBILE_BREAKPOINT_PX))
                            .clamp(0.0, f64::from(u16::MAX)) as u16,
                    );
                }
                if mark_document_hydrated() {
                    is_hydrated.set(true);
                }
            }
        });
        let listener = window_event_listener(leptos::ev::resize, move |_| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let width = window
                .inner_width()
                .ok()
                .and_then(|width| width.as_f64())
                .unwrap_or(f64::from(SIDEBAR_MOBILE_BREAKPOINT_PX));
            let width = width.clamp(0.0, f64::from(u16::MAX)) as u16;
            viewport_width.set(width);
            if width >= SIDEBAR_MOBILE_BREAKPOINT_PX && is_drawer_open.get_untracked() {
                if let Some(dialog) = drawer.get_untracked() {
                    dialog.close();
                }
            }
        });
        on_cleanup(move || listener.remove());
    }

    view! {
        <a class="skip-link" href="#main">{move || locale.get().select("跳转到正文", "Skip to content")}</a>
        <div class="site-shell" style=move || format!("--sidebar-width: {}px", preferences.get().sidebar_width_px)
            on:click=move |event| {
                #[cfg(feature = "hydrate")]
                {
                    if !super::navigation_behavior::is_unmodified_click(&event) {
                        return;
                    }
                    let is_navigation_link = event.target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .is_some_and(|target| is_main_navigation_link(&target));
                    if is_navigation_link {
                        let main = web_sys::window().and_then(|window| window.document())
                            .and_then(|document| document.get_element_by_id("main"))
                            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
                        focus_element_for_shell(main);
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = event;
            }>
            <Sidebar viewport_width is_hydrated/>
            <div node_ref=workspace class="site-workspace">
                <Topbar is_hydrated is_drawer_open drawer trigger/>
                <div class="mobile-fallback-nav">
                    <SidebarNavigation show_writing_link=true/>
                </div>
                <div class="content-layout" class:has-current-directory=move || has_article_directory(&pathname.get())>
                    <main id="main" class="site-content" tabindex="-1">{children()}</main>
                    <CurrentDirectory/>
                </div>
                <footer class="site-footer">
                    <div class="footer-top"><span class="footer-glyph" aria-hidden="true"><AsteriskGlyph/></span><span>"KEEP BUILDING"<br/>"KEEP QUESTIONING."</span></div>
                    <div class="footer-bottom"><span>"© 2026 MCB-SMART-BOY "<span class="footer-separator">"/"</span>" BUILT WITH RUST"</span><span>
                        <a href=LandingSection::Home.href() target="_self">{move || LandingSection::Home.label(locale.get())}</a>
                        <a href=LandingSection::Focus.href() target="_self">{move || LandingSection::Focus.label(locale.get())}</a>
                        <a href=LandingSection::About.href() target="_self">{move || LandingSection::About.label(locale.get())}</a>
                        <a href="/writing/">{move || locale.get().writing()}</a>
                        <a href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">"GITHUB ↗"</a>
                    </span></div>
                </footer>
            </div>
        </div>
        <dialog
            node_ref=drawer
            id="mobile-drawer"
            class="mobile-drawer"
            aria-label=move || locale.get().select("主导航", "Main navigation")
            on:close=move |_| {
                is_drawer_open.set(false);
                #[cfg(feature = "hydrate")]
                {
                    if let Some(section) = landing_focus_after_close.get_untracked()
                        .filter(|_| pathname.get_untracked() == "/")
                    {
                        focus_landing_section(section);
                    } else if focus_main_after_close.get_untracked()
                        || viewport_width.get_untracked() >= SIDEBAR_MOBILE_BREAKPOINT_PX
                    {
                        let main = web_sys::window().and_then(|window| window.document())
                            .and_then(|document| document.get_element_by_id("main"))
                            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
                        focus_element_for_shell(main);
                    } else {
                        focus_element_for_shell(trigger.get_untracked().map(|element| element.into()));
                    }
                }
                focus_main_after_close.set(false);
                landing_focus_after_close.set(None);
            }
            on:click=move |event| {
                #[cfg(feature = "hydrate")]
                {
                    if event.target().is_some_and(|target| target.dyn_into::<web_sys::HtmlDialogElement>().is_ok()) {
                        if let Some(dialog) = drawer.get_untracked() { dialog.close(); }
                        return;
                    }
                    if !super::navigation_behavior::is_unmodified_click(&event) {
                        return;
                    }
                    let internal_href = event.target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .and_then(|target| target.closest("a[href]:not([data-sidebar-level]):not([data-reading-branch])").ok().flatten())
                        .and_then(|link| link.get_attribute("href"))
                        .filter(|href| href.starts_with('/'));
                    if let Some(href) = internal_href {
                        let landing_target = landing_section_from_href(&href);
                        landing_focus_after_close.set(landing_target);
                        focus_main_after_close.set(landing_target.is_none());
                        if let Some(dialog) = drawer.get_untracked() { dialog.close(); }
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                let _ = event;
            }
        >
            <div class="drawer-panel">
                <button class="drawer-close topbar-button" type="button" aria-label=move || locale.get().select("关闭导航菜单", "Close navigation menu") on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    if let Some(dialog) = drawer.get_untracked() { dialog.close(); }
                }><Icon kind=IconKind::Close/></button>
                <SidebarBrand is_hydrated is_desktop=false/>
                <ReadingNavigation is_hydrated is_visible=is_mobile_visible reading_request=mobile_reading_request id_prefix="mobile"/>
            </div>
        </dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::{SIDEBAR_MAX_PX, Theme};

    #[test]
    fn parse_browser_preferences_valid_saved_values_restores_all_fields() {
        let cookies = "session=ignored; mcb-ui=dark:1:999; mcb-lang=en";
        let (preferences, locale) = parse_browser_preferences(Some(cookies));

        assert_eq!(preferences.theme, Theme::Dark);
        assert!(preferences.is_sidebar_collapsed);
        assert_eq!(preferences.sidebar_width_px, SIDEBAR_MAX_PX);
        assert_eq!(locale, Locale::En);
    }
}
