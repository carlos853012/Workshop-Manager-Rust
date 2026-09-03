use dioxus::prelude::*;

use crate::components::molecules::card::Card;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Reports() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    rsx! {
        AppShell { title: "Reportes".to_string(), active_route: Route::Reports {},
            Card { title: "Reportes".to_string(),
                p { class: "text-muted", "Próximamente: reportes de clientes e historial." }
            }
        }
    }
}
