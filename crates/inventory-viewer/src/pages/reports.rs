use dioxus::prelude::*;

use crate::components::molecules::card::Card;
use crate::pages::layout::{require_auth, AppShell};

#[component]
pub fn Reports() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    rsx! {
        AppShell { title: "Reportes".to_string(), active_route: "/reports".to_string(),
            Card { title: "Reportes".to_string(),
                p { class: "text-muted", "Próximamente: reportes de clientes e historial." }
            }
        }
    }
}
