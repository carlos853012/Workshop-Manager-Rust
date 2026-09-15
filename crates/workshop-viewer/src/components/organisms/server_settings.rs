use dioxus::prelude::*;

use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;
use crate::config::{config, save_config, ViewerConfig};

#[component]
pub fn ConnectionSettingsButton(class: Option<String>) -> Element {
    let mut show_settings = use_signal(|| false);

    rsx! {
        Button {
            class: class,
            variant: ButtonVariant::Ghost,
            title: "Configuración de conexión",
            onclick: move |_| show_settings.set(true),
            "⚙️"
        }
        ConnectionSettings {
            show: *show_settings.read(),
            on_close: move |_| show_settings.set(false),
        }
    }
}

#[component]
pub fn ConnectionSettings(show: bool, on_close: EventHandler<()>) -> Element {
    let initial = config();
    let mut base_url = use_signal(|| initial.server.base_url.clone());
    let mut api_key = use_signal(|| initial.server.api_key.clone());
    let mut device_key = use_signal(|| initial.server.device_key.clone());
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);
    let mut testing = use_signal(|| false);
    let mut test_result = use_signal(|| None::<String>);
    let mut test_ok = use_signal(|| false);

    let on_save = move |_| {
        let raw = base_url.read().trim().trim_end_matches('/').to_string();
        if raw.is_empty() {
            error.set(Some(
                "La dirección del servidor es obligatoria.".to_string(),
            ));
            return;
        }
        let base_url_value = crate::config::resolve_base_url(&raw);

        let new_config = ViewerConfig {
            server: crate::config::ServerSection {
                base_url: base_url_value,
                api_key: api_key.read().trim().to_string(),
                device_key: device_key.read().trim().to_string(),
                tls_accept_invalid_certs: initial.server.tls_accept_invalid_certs,
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

    let on_test = move |_| {
        let raw = base_url.read().trim().trim_end_matches('/').to_string();
        if raw.is_empty() {
            test_result.set(Some(
                "Ingresa la dirección del servidor primero.".to_string(),
            ));
            test_ok.set(false);
            return;
        }
        let url = crate::config::resolve_base_url(&raw);

        testing.set(true);
        test_result.set(None);
        test_ok.set(false);

        let accept_invalid = initial.server.tls_accept_invalid_certs;
        spawn(async move {
            let client = match reqwest::Client::builder()
                .danger_accept_invalid_certs(accept_invalid)
                .timeout(std::time::Duration::from_secs(10))
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    test_result.set(Some(format!("Error creando cliente HTTP: {e}")));
                    test_ok.set(false);
                    testing.set(false);
                    return;
                }
            };

            let health_url = format!("{}/health", url);
            match client.get(&health_url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        test_result.set(Some(format!("Conexion exitosa (HTTP {status})")));
                        test_ok.set(true);
                    } else {
                        let body = resp.text().await.unwrap_or_default();
                        test_result.set(Some(format!("Servidor respondio HTTP {status}: {body}")));
                        test_ok.set(false);
                    }
                }
                Err(e) => {
                    let msg = if e.is_timeout() {
                        "Tiempo de espera agotado. Verifica la IP y que el servidor este encendido."
                            .to_string()
                    } else if e.is_connect() {
                        format!("No se pudo conectar: {e}. Verifica la IP y el firewall.")
                    } else if e.to_string().contains("certificate") || e.to_string().contains("tls")
                    {
                        format!("Error TLS: {e}. Verifica que la URL use https://")
                    } else {
                        format!("Error de red: {e}")
                    };
                    test_result.set(Some(msg));
                    test_ok.set(false);
                }
            }
            testing.set(false);
        });
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
            if let Some(message) = test_result.read().as_ref() {
                div {
                    class: if *test_ok.read() {
                        "alert alert-success mb-md"
                    } else {
                        "alert alert-danger mb-md"
                    },
                    "{message}"
                }
            }
            Input {
                label: Some("IP del servidor".to_string()),
                placeholder: Some("192.168.1.100".to_string()),
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
            div { class: "mt-md",
                Button {
                    variant: ButtonVariant::Ghost,
                    disabled: *testing.read(),
                    onclick: on_test,
                    if *testing.read() { "Probando..." } else { "Probar conexión" }
                }
            }
        }
    }
}
