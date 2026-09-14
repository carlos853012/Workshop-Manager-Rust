use dioxus::prelude::*;
use dioxus_router::prelude::*;
use workshop_common::UserRole;

use crate::app_state::{use_auth, use_tabs};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::icons::IconName;
use crate::routes::Route;
use crate::theme::use_theme;

#[component]
pub fn Header(title: String, active_route: Route) -> Element {
    let mut theme = use_theme();
    let auth = use_auth();

    rsx! {
        header { class: "header",
            div { class: "header-navigation",
                h1 { class: "header-title", "{title}" }
                if auth.is_trial() {
                    span { class: "trial-badge", "TRIAL" }
                }
                Tabs { active_route: active_route }
            }
            div { class: "header-actions",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_evt| theme.toggle(),
                    "🌓"
                }
            }
        }
    }
}

#[component]
fn Tabs(active_route: Route) -> Element {
    let tabs_state = use_tabs();
    let navigator = use_navigator();

    rsx! {
        nav { class: "tabs", aria_label: "Rutas abiertas",
            for tab in tabs_state.tabs.read().iter() {
                div { class: "tab",
                    Link {
                        class: if tab.route == active_route { "tab-link active" } else { "tab-link" },
                        to: tab.route.clone(),
                        title: "Ir a {tab.title}",
                        "{tab.title}"
                    }
                    if tab.route != (Route::Dashboard {}) {
                        button {
                            class: "tab-close",
                            title: "Cerrar {tab.title}",
                            onclick: {
                                let route_to_close = tab.route.clone();
                                let active_route = active_route.clone();
                                let mut tabs = tabs_state.tabs;
                                move |_| {
                                    let current_tabs = tabs.read().clone();
                                    let Some(closed_index) = current_tabs
                                        .iter()
                                        .position(|open_tab| open_tab.route == route_to_close)
                                    else {
                                        return;
                                    };

                                    tabs.write().retain(|open_tab| open_tab.route != route_to_close);

                                    if route_to_close == active_route {
                                        let fallback_index = closed_index.saturating_sub(1);
                                        let fallback_route = current_tabs
                                            .iter()
                                            .filter(|open_tab| open_tab.route != route_to_close)
                                            .nth(fallback_index)
                                            .map(|open_tab| open_tab.route.clone())
                                            .unwrap_or(Route::Dashboard {});
                                        navigator.push(fallback_route);
                                    }
                                }
                            },
                            "x"
                        }
                    }
                }
            }
        }
    }
}

/// Item de navegación para el sidebar.
#[derive(Clone, PartialEq)]
pub struct NavItem {
    pub label: String,
    pub route: Route,
    pub icon: IconName,
}

#[component]
pub fn Sidebar(
    items: Vec<NavItem>,
    active_route: Route,
    collapsed: bool,
    user_name: Option<String>,
    user_display_name: Option<String>,
    user_role: Option<UserRole>,
    workshop_name: Option<String>,
    workshop_city: Option<String>,
    on_logout: Option<EventHandler<()>>,
) -> Element {
    let sidebar_class = if collapsed {
        "sidebar collapsed"
    } else {
        "sidebar"
    };
    let avatar_initial = match user_display_name
        .as_deref()
        .or(user_name.as_deref())
        .and_then(|name| name.chars().next())
    {
        Some(initial) => initial.to_uppercase().to_string(),
        None => "U".to_string(),
    };

    rsx! {
        aside { class: "{sidebar_class}",
            div { class: "sidebar-logo",
                div { class: "sidebar-user",
                    span { class: "user-avatar", "{avatar_initial}" }
                    div { class: "user-details",
                        if let Some(name) = workshop_name {
                            strong { "{name}" }
                        }
                        if let Some(name) = user_display_name {
                            span { class: "user-name", "{name}" }
                        }
                        if let Some(role) = user_role {
                            span { class: "user-role", "{crate::i18n::translate_role(&role)}" }
                        }
                    }
                }
            }
            nav { class: "sidebar-nav",
                for item in items {
                    SidebarLink {
                        item: item.clone(),
                        active: item.route == active_route,
                    }
                }
            }
            if on_logout.is_some() {
                div { class: "sidebar-footer",
                    Button {
                        class: Some("sidebar-logout".to_string()),
                        variant: ButtonVariant::Ghost,
                        title: "Cerrar sesión",
                        onclick: move |_evt| {
                            if let Some(handler) = on_logout.as_ref() {
                                handler.call(());
                            }
                        },
                        {IconName::Logout.render()}
                        span { class: "sidebar-logout-label", "Salir" }
                    }
                    if let Some(name) = user_name {
                        span { class: "sidebar-footer-email", "{name}" }
                    }
                    if let Some(city) = workshop_city {
                        span { class: "sidebar-footer-email", "{city}" }
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarLink(item: NavItem, active: bool) -> Element {
    let class = if active {
        "sidebar-link active"
    } else {
        "sidebar-link"
    };
    rsx! {
        Link {
            class: "{class}",
            title: "{item.label}",
            to: item.route.clone(),
            {item.icon.render()}
            span { "{item.label}" }
        }
    }
}
