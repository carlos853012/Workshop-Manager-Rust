use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::config::{config, save_config, ViewerConfig};
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Dashboard() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let mut show_settings = use_signal(|| false);

    rsx! {
        AppShell { title: "Dashboard".to_string(), active_route: Route::Dashboard {},
            div { class: "dashboard-toolbar",
                h2 { "Inicio" }
                Button {
                    variant: ButtonVariant::Ghost,
                    title: "Configuración de conexión",
                    onclick: move |_| show_settings.set(true),
                    {IconName::Cog.render()}
                }
            }
            div { class: "grid grid-4",
                DashboardCard { title: "Productos".to_string(), value: "--".to_string(), icon: IconName::Package }
                DashboardCard { title: "Ventas".to_string(), value: "--".to_string(), icon: IconName::ShoppingCart }
                DashboardCard { title: "Reparaciones".to_string(), value: "--".to_string(), icon: IconName::Wrench }
                DashboardCard { title: "Proveedores".to_string(), value: "--".to_string(), icon: IconName::Truck }
            }
            div { class: "mt-lg",
                Card { title: "Próximamente".to_string(),
                    p { class: "text-muted", "El dashboard con datos reales se conectará a /api/analytics/dashboard en la siguiente iteración." }
                }
            }
            ConnectionSettings {
                show: *show_settings.read(),
                on_close: move |_| show_settings.set(false),
            }
        }
    }
}

#[component]
fn ConnectionSettings(show: bool, on_close: EventHandler<()>) -> Element {
    let initial = config();
    let mut base_url = use_signal(|| initial.server.base_url.clone());
    let mut api_key = use_signal(|| initial.server.api_key.clone());
    let mut device_key = use_signal(|| initial.server.device_key.clone());
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);

    let on_save = move |_| {
        let base_url_value = base_url.read().trim().trim_end_matches('/').to_string();
        if base_url_value.is_empty() {
            error.set(Some(
                "La dirección del servidor es obligatoria.".to_string(),
            ));
            return;
        }

        let new_config = ViewerConfig {
            server: crate::config::ServerSection {
                base_url: base_url_value,
                api_key: api_key.read().trim().to_string(),
                device_key: device_key.read().trim().to_string(),
            },
        };

        match save_config(new_config) {
            Ok(()) => {
                error.set(None);
                saved.set(true);
            }
            Err(message) => error.set(Some(message)),
        }
    };

    rsx! {
        Modal {
            show: show,
            title: "Configuración de conexión".to_string(),
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                Button {
                    class: Some("cancel-button".to_string()),
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_save,
                    "Guardar"
                }
            },
            if let Some(message) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{message}" }
            }
            if *saved.read() {
                div { class: "alert alert-success mb-md", "Configuración guardada." }
            }
            Input {
                label: Some("Dirección del servidor".to_string()),
                placeholder: Some("https://192.168.1.100:8443".to_string()),
                value: base_url.read().clone(),
                oninput: move |event: FormEvent| base_url.set(event.value().clone()),
                required: true,
            }
            Input {
                label: Some("API key".to_string()),
                r#type: "password".to_string(),
                value: api_key.read().clone(),
                oninput: move |event: FormEvent| api_key.set(event.value().clone()),
            }
            Input {
                label: Some("Device key".to_string()),
                r#type: "password".to_string(),
                value: device_key.read().clone(),
                oninput: move |event: FormEvent| device_key.set(event.value().clone()),
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
        navigator.push(Route::Setup {});
    }

    rsx! {}
}

#[component]
fn DashboardCard(title: String, value: String, icon: IconName) -> Element {
    rsx! {
        Card {
            div { class: "flex items-center gap-md",
                span { class: "text-2xl", {icon.render()} }
                div {
                    p { class: "text-muted text-sm", "{title}" }
                    p { class: "text-xl font-semibold", "{value}" }
                }
            }
        }
    }
}
