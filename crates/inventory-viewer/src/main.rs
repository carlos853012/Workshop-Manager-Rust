use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("../index.css")} }
        div { class: "container",
            h1 { "WorkshopManager" }
            p { "Sistema de Gestión de Taller" }
            p { "Versión 0.1.0" }
        }
    }
}
