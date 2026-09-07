use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app_state::{use_auth, use_tabs, OpenTab};
use crate::components::organisms::header::Sidebar;
use crate::components::organisms::header::{Header, NavItem};
use crate::icons::IconName;
use crate::routes::Route;

/// Layout con sidebar y header para páginas autenticadas.
#[component]
pub fn AppShell(children: Element, title: String, active_route: Route) -> Element {
    let mut auth = use_auth();
    let navigator = use_navigator();
    let mut tabs_state = use_tabs();
    let mut sidebar_collapsed = use_signal(|| false);

    let route_for_tab = active_route.clone();
    let title_for_tab = title.clone();
    use_effect(move || {
        let mut tabs = tabs_state.tabs.write();
        if !tabs.iter().any(|tab| tab.route == route_for_tab) {
            tabs.push(OpenTab {
                title: title_for_tab.clone(),
                route: route_for_tab.clone(),
            });
        }
    });

    use_future(move || async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        sidebar_collapsed.set(true);
    });

    let nav_items = vec![
        NavItem {
            label: "Dashboard".to_string(),
            route: Route::Dashboard {},
            icon: IconName::Home,
        },
        NavItem {
            label: "Productos".to_string(),
            route: Route::Products {},
            icon: IconName::Package,
        },
        NavItem {
            label: "POS".to_string(),
            route: Route::Pos {},
            icon: IconName::ShoppingCart,
        },
        NavItem {
            label: "Ventas".to_string(),
            route: Route::Sales {},
            icon: IconName::Cash,
        },
        NavItem {
            label: "Reparaciones".to_string(),
            route: Route::Repairs {},
            icon: IconName::Wrench,
        },
        NavItem {
            label: "Proveedores".to_string(),
            route: Route::Suppliers {},
            icon: IconName::Truck,
        },
        NavItem {
            label: "Reportes".to_string(),
            route: Route::Reports {},
            icon: IconName::ChartBar,
        },
        NavItem {
            label: "Usuarios".to_string(),
            route: Route::Users {},
            icon: IconName::Users,
        },
        NavItem {
            label: "Claves Dispositivo".to_string(),
            route: Route::DeviceKeys {},
            icon: IconName::Key,
        },
    ];

    let user_name = auth.user_email.read().clone();
    let user_display_name = auth.user_display_name.read().clone();
    let user_role = auth.user_role.read().clone();
    let workshop = auth.workshop.read().clone();

    let on_logout = move |_| {
        auth.logout();
        navigator.push(Route::Login {});
    };

    rsx! {
        div { class: "app-shell",
            Sidebar {
                items: nav_items,
                active_route: active_route.clone(),
                collapsed: sidebar_collapsed(),
                user_name: user_name.clone(),
                user_display_name,
                user_role,
                workshop_name: workshop.as_ref().map(|value| value.name.clone()),
                workshop_city: workshop.as_ref().map(|value| value.city.clone()),
                on_logout: Some(EventHandler::new(on_logout)),
            }
            div { class: "main-content",
                Header {
                    title: title.clone(),
                    active_route: active_route.clone(),
                }
                main { class: "page", {children} }
            }
        }
    }
}

/// Redirige a login si no hay sesión.
pub fn require_auth() -> Option<()> {
    let auth = use_auth();
    if auth.is_authenticated() {
        Some(())
    } else {
        let navigator = use_navigator();
        navigator.push(Route::Login {});
        None
    }
}
