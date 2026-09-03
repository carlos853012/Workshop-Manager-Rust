mod api;
mod app_state;
mod components;
mod config;
mod icons;
mod pages;
mod routes;
mod theme;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app_state::AuthProvider;
use crate::routes::Route;
use crate::theme::ThemeProvider;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("../index.css")} }
        ThemeProvider {
            AuthProvider {
                Router::<Route> {}
            }
        }
    }
}
