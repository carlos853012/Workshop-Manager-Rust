use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app_state::use_auth;
use crate::components::molecules::card::Card;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Dashboard() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    rsx! {
        AppShell { title: "Dashboard".to_string(), active_route: "/dashboard".to_string(),
            div { class: "grid grid-4",
                DashboardCard { title: "Productos".to_string(), value: "--".to_string(), icon: "📦" }
                DashboardCard { title: "Ventas".to_string(), value: "--".to_string(), icon: "🛒" }
                DashboardCard { title: "Reparaciones".to_string(), value: "--".to_string(), icon: "🔧" }
                DashboardCard { title: "Proveedores".to_string(), value: "--".to_string(), icon: "🚚" }
            }
            div { class: "mt-lg",
                Card { title: "Próximamente".to_string(),
                    p { class: "text-muted", "El dashboard con datos reales se conectará a /api/analytics/dashboard en la siguiente iteración." }
                }
            }
        }
    }
}

#[component]
pub fn Root() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();

    if auth.is_authenticated() {
        navigator.push(Route::Dashboard {});
    } else {
        navigator.push(Route::Login {});
    }

    rsx! {}
}

#[component]
fn DashboardCard(title: String, value: String, icon: String) -> Element {
    rsx! {
        Card {
            div { class: "flex items-center gap-md",
                span { class: "text-2xl", "{icon}" }
                div {
                    p { class: "text-muted text-sm", "{title}" }
                    p { class: "text-xl font-semibold", "{value}" }
                }
            }
        }
    }
}
