use dioxus::prelude::*;
use inventory_common::dto::ClientSearchResult;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn ServiceCertificatePage() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let mut search_query = use_signal(|| "".to_string());
    let mut results = use_signal(Vec::<ClientSearchResult>::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let generating = use_signal(|| false);
    let success_msg = use_signal(|| None::<String>);

    let download_cert = move |(email, name, plate): (Option<String>, Option<String>, Option<String>)| {
        let client = auth.api_client();
        let mut generating_set = generating;
        let mut error_set = error;
        let mut success_set = success_msg;
        generating_set.set(true);
        error_set.set(None);
        success_set.set(None);
        let mut auth = auth;
        spawn(async move {
            if let Some(api) = client {
                match api.download_certificate(email.as_deref(), name.as_deref(), plate.as_deref()).await {
                    Ok(pdf_bytes) => {
                        let label = email.as_deref().or(name.as_deref()).unwrap_or("cliente");
                        let filename = format!("certificado_{}.pdf", label.replace('@', "_at_"));
                        let path = std::path::PathBuf::from(&filename);
                        if let Err(e) = std::fs::write(&path, &pdf_bytes) {
                            error_set.set(Some(format!("Error al guardar: {}", e)));
                        } else {
                            success_set
                                .set(Some(format!("Certificado guardado en {}", path.display())));
                        }
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            } else {
                error_set.set(Some("No hay cliente API".to_string()));
            }
            generating_set.set(false);
        });
    };

    rsx! {
        AppShell { title: "Certificado de Servicios".to_string(), active_route: Route::ServiceCertificatePage {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "alert alert-success mb-md", "{msg}" }
            }

            Card {
                div { class: "cert-search-wrapper",
                    input {
                        class: "cert-search-input",
                        r#type: "text",
                        value: "{search_query}",
                        placeholder: "Patente, nombre o email del cliente...",
                        oninput: move |evt: Event<FormData>| search_query.set(evt.value()),
                        onkeydown: {
                            let search_query = search_query;
                            let mut auth = auth;
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter {
                                    let q = search_query.read().trim().to_string();
                                    if q.is_empty() { return; }
                                    loading.set(true);
                                    error.set(None);
                                    let client = auth.api_client();
                                    spawn(async move {
                                        if let Some(api) = client {
                                            match api.client_search(&q).await {
                                                Ok(data) => { results.set(data); }
                                                Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                                                    auth.logout();
                                                    navigator.push(Route::Login {});
                                                }
                                                Err(e) => { error.set(Some(e.user_message().to_string())); }
                                            }
                                        } else {
                                            error.set(Some("No hay cliente API".to_string()));
                                        }
                                        loading.set(false);
                                    });
                                }
                            }
                        },
                    }
                }
            }

            if *loading.read() {
                div { class: "empty-state mt-lg", Spinner {} }
            } else if !results.read().is_empty() {
                for result in results.read().iter() {
                    Card {
                        div { class: "mb-md",
                            div { class: "flex items-center gap-md mb-sm",
                                h3 { class: "text-lg font-semibold",
                                    "{result.customer_name.as_deref().unwrap_or(\"Sin nombre\")}"
                                }
                                if let Some(ref email) = result.customer_email {
                                    span { class: "badge badge-info", "{email}" }
                                }
                                if let Some(ref phone) = result.customer_phone {
                                    span { class: "text-muted text-sm", "Tel: {phone}" }
                                }
                            }
                        }
                        if result.vehicles.is_empty() {
                            div { class: "empty-state",
                                {IconName::InformationCircle.render()}
                                p { "No se encontraron vehículos para este cliente" }
                            }
                        } else {
                            div { class: "data-table-wrapper",
                                table { class: "data-table",
                                    thead {
                                        tr {
                                            th { "Vehículo" }
                                            th { "Patente" }
                                            th { class: "text-right", "Reparaciones" }
                                            th { "Última reparación" }
                                            th { class: "text-right", "Acciones" }
                                        }
                                    }
                                    tbody {
                                        for vehicle in result.vehicles.iter() {
                                            tr {
                                                td {
                                                    "{vehicle.vehicle.as_deref().unwrap_or(\"Vehículo\")}"
                                                }
                                                td {
                                                    if let Some(ref plate) = vehicle.license_plate {
                                                        span { class: "badge badge-info", "{plate}" }
                                                    } else {
                                                        span { class: "text-muted", "-" }
                                                    }
                                                }
                                                td { class: "text-right",
                                                    "{vehicle.total_repairs}"
                                                }
                                                td { class: "text-muted",
                                                    "{vehicle.last_repair_date.map(|d| d.format(\"%d-%m-%Y\").to_string()).unwrap_or_else(|| \"-\".to_string())}"
                                                }
                                                td { class: "text-right",
                                                    Button {
                                                        variant: ButtonVariant::Primary,
                                                        onclick: {
                                                            let email = result.customer_email.clone();
                                                            let name = result.customer_name.clone();
                                                            let plate = vehicle.license_plate.clone();
                                                            move |_| download_cert((email.clone(), name.clone(), plate.clone()))
                                                        },
                                                        disabled: *generating.read(),
                                                        if *generating.read() {
                                                            Spinner {}
                                                        } else {
                                                            {IconName::DocumentText.render()}
                                                            " Generar certificado"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if !search_query.read().trim().is_empty() && !*loading.read() {
                div { class: "empty-state mt-lg",
                    {IconName::Search.render()}
                    p { "No se encontraron resultados para \"{search_query}\"" }
                }
            }
        }
    }
}
