use dioxus::prelude::*;

use crate::components::molecules::card::Card;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Users() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    rsx! {
        AppShell { title: "Usuarios".to_string(), active_route: Route::Users {},
            Card { title: "Usuarios".to_string(),
                p { class: "text-muted", "Próximamente: gestión de usuarios." }
            }
        }
    }
}
