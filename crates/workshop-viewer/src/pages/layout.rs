use dioxus::prelude::*;
use dioxus_router::prelude::*;
use workshop_common::UserRole;

use crate::app_state::{use_auth, use_sidebar, use_tabs, OpenTab};
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
    let mut sidebar = use_sidebar();

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

    let user_role = auth.user_role.read().clone();
    let role = user_role.as_ref().cloned().unwrap_or(UserRole::Seller);

    let mut nav_items = vec![
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
    ];

    if matches!(role, UserRole::Admin | UserRole::Seller) {
        nav_items.push(NavItem {
            label: "POS".to_string(),
            route: Route::Pos {},
            icon: IconName::ShoppingCart,
        });
        nav_items.push(NavItem {
            label: "Ventas".to_string(),
            route: Route::Sales {},
            icon: IconName::Cash,
        });
    }

    nav_items.push(NavItem {
        label: "Reparaciones".to_string(),
        route: Route::Repairs {},
        icon: IconName::Wrench,
    });

    if matches!(role, UserRole::Admin | UserRole::Seller) {
        nav_items.push(NavItem {
            label: "Proveedores".to_string(),
            route: Route::Suppliers {},
            icon: IconName::Truck,
        });
    }

    nav_items.push(NavItem {
        label: "Reportes".to_string(),
        route: Route::Reports {},
        icon: IconName::ChartBar,
    });
    nav_items.push(NavItem {
        label: "Certificado Servicios".to_string(),
        route: Route::ServiceCertificatePage {},
        icon: IconName::DocumentText,
    });

    if matches!(role, UserRole::Admin) {
        nav_items.push(NavItem {
            label: "Usuarios".to_string(),
            route: Route::Users {},
            icon: IconName::Users,
        });
        nav_items.push(NavItem {
            label: "Claves Dispositivo".to_string(),
            route: Route::DeviceKeys {},
            icon: IconName::Key,
        });
    }

    let user_name = auth.user_email.read().clone();
    let user_display_name = auth.user_display_name.read().clone();
    let workshop = auth.workshop.read().clone();

    let on_logout = move |_| {
        tabs_state.tabs.write().clear();
        tabs_state.tabs.write().push(OpenTab {
            title: "Dashboard".to_string(),
            route: Route::Dashboard {},
        });
        sidebar.collapsed.set(false);
        sidebar.hovered.set(false);
        auth.logout();
        navigator.push(Route::Login {});
    };

    rsx! {
        div { class: "app-shell",
            Sidebar {
                items: nav_items,
                active_route: active_route.clone(),
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
