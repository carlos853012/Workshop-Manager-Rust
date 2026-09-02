use dioxus::prelude::*;

use crate::components::atoms::button::{Button, ButtonVariant};
use crate::icons::IconName;
use crate::theme::use_theme;

#[component]
pub fn Header(
    title: String,
    #[props(default = None)] user_name: Option<String>,
    on_logout: Option<EventHandler<()>>,
) -> Element {
    let mut theme = use_theme();

    rsx! {
        header { class: "header",
            h1 { class: "header-title", "{title}" }
            div { class: "header-actions",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_evt| theme.toggle(),
                    "🌓"
                }
                if let Some(name) = user_name {
                    span { class: "text-muted", "{name}" }
                }
                if on_logout.is_some() {
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_evt| {
                            if let Some(handler) = on_logout.as_ref() {
                                handler.call(());
                            }
                        },
                        "Salir"
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
    pub route: String,
    pub icon: IconName,
}

#[component]
pub fn Sidebar(
    items: Vec<NavItem>,
    active_route: String,
) -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-logo",
                "🔧 WorkshopManager"
            }
            nav { class: "sidebar-nav",
                for item in items {
                    SidebarLink {
                        item: item.clone(),
                        active: item.route == active_route,
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarLink(item: NavItem, active: bool) -> Element {
    let class = if active { "sidebar-link active" } else { "sidebar-link" };
    rsx! {
        a {
            class: "{class}",
            href: "{item.route}",
            span { "{item.icon.as_emoji()}" }
            span { "{item.label}" }
        }
    }
}
