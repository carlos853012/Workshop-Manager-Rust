use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app_state::use_auth;
use crate::components::organisms::header::Sidebar;
use crate::components::organisms::header::{Header, NavItem};
use crate::icons::IconName;
use crate::routes::Route;

/// Layout con sidebar y header para páginas autenticadas.
#[component]
pub fn AppShell(children: Element, title: String, active_route: String) -> Element {
    let mut auth = use_auth();
    let navigator = use_navigator();

    let nav_items = vec![
        NavItem {
            label: "Dashboard".to_string(),
            route: "/dashboard".to_string(),
            icon: IconName::Home,
        },
        NavItem {
            label: "Productos".to_string(),
            route: "/products".to_string(),
            icon: IconName::Package,
        },
        NavItem {
            label: "Ventas".to_string(),
            route: "/sales".to_string(),
            icon: IconName::ShoppingCart,
        },
        NavItem {
            label: "Reparaciones".to_string(),
            route: "/repairs".to_string(),
            icon: IconName::Wrench,
        },
        NavItem {
            label: "Proveedores".to_string(),
            route: "/suppliers".to_string(),
            icon: IconName::Truck,
        },
        NavItem {
            label: "Reportes".to_string(),
            route: "/reports".to_string(),
            icon: IconName::DocumentText,
        },
        NavItem {
            label: "Usuarios".to_string(),
            route: "/users".to_string(),
            icon: IconName::Users,
        },
    ];

    let user_name = auth.user_email.read().clone();

    let on_logout = move |_| {
        auth.logout();
        navigator.push(Route::Login {});
    };

    rsx! {
        div { class: "app-shell",
            Sidebar { items: nav_items, active_route: active_route.clone() }
            div { class: "main-content",
                Header { title: title.clone(), user_name: user_name.clone(), on_logout: Some(EventHandler::new(on_logout)) }
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
