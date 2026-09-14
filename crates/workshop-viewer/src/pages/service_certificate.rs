use dioxus::prelude::*;
use workshop_common::{Repair};

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::certificate_detail_modal::CertificateDetailModal;
use crate::icons::IconName;
use crate::i18n;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn ServiceCertificatePage() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let mut auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let mut search_query = use_signal(String::new);
    let all_repairs = use_signal(Vec::<Repair>::new);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let generating = use_signal(|| false);
    let success_msg = use_signal(|| None::<String>);

    let mut show_detail = use_signal(|| false);
    let mut detail_repair_id = use_signal(|| None::<uuid::Uuid>);

    use_effect(move || {
        let client = auth.api_client();
        let mut loading_set = loading;
        let mut error_set = error;
        let mut repairs_set = all_repairs;
        loading_set.set(true);
        error_set.set(None);
        spawn(async move {
            if let Some(api) = client {
                match api.list_repairs(1, 100).await {
                    Ok(page) => {
                        repairs_set.set(page.items);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                        return;
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            }
            loading_set.set(false);
        });
    });

    let filtered: Vec<Repair> = {
        let q = search_query.read().to_lowercase();
        let q = q.trim();
        if q.is_empty() {
            all_repairs.read().clone()
        } else {
            all_repairs
                .read()
                .iter()
                .filter(|r| {
                    let name = r.customer_name.as_deref().unwrap_or("").to_lowercase();
                    let plate = r.license_plate.as_deref().unwrap_or("").to_lowercase();
                    let vehicle = r.vehicle.as_deref().unwrap_or("").to_lowercase();
                    let desc = r.description.as_deref().unwrap_or("").to_lowercase();
                    name.contains(q)
                        || plate.contains(q)
                        || vehicle.contains(q)
                        || desc.contains(q)
                })
                .cloned()
                .collect()
        }
    };

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
                            success_set.set(Some(format!("Certificado guardado en {}", path.display())));
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
                div { class: "alert alert-success mt-sm", "{msg}" }
            }
            div { class: "cert-search-wrapper",
                    input {
                        class: "cert-search-input",
                        r#type: "text",
                        value: "{search_query}",
                        placeholder: "Filtrar por cliente, patente, vehículo o descripción...",
                        oninput: move |evt: Event<FormData>| search_query.set(evt.value()),
                    }
                }
            Card {
                title: "Reparaciones del taller".to_string(),


                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    div { class: "products-table",
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "Cliente" }
                                    th { "Vehículo" }
                                    th { "Patente" }
                                    th { "Estado" }
                                    th { "Fecha" }
                                    th { class: "col-actions", "" }
                                }
                            }
                            tbody {
                                if filtered.is_empty() {
                                    tr {
                                        td {
                                            colspan: "6",
                                            div { class: "empty-state",
                                                {IconName::Search.render()}
                                                p { "No se encontraron reparaciones" }
                                            }
                                        }
                                    }
                                } else {
                                    for r in filtered.iter() {
                                        tr {
                                            td { "{r.customer_name.as_deref().unwrap_or(\"-\")}" }
                                            td { "{r.vehicle.as_deref().unwrap_or(\"-\")}" }
                                            td {
                                                if let Some(ref plate) = r.license_plate {
                                                    "{plate.to_uppercase()}"
                                                } else {
                                                    "-"
                                                }
                                            }
                                             td {
                                                 "{i18n::translate_repair_status(&r.status)}"
                                             }
                                            td { class: "text-muted",
                                                "{r.created_at.format(\"%d-%m-%Y\").to_string()}"
                                            }
                                            td { class: "col-actions",
                                                div { class: "table-actions",
                                                    button {
                                                        class: "btn-icon btn-edit",
                                                        title: "Ver detalle",
                                                        onclick: {
                                                            let rid = r.id;
                                                            move |e: Event<MouseData>| {
                                                                e.stop_propagation();
                                                                detail_repair_id.set(Some(rid));
                                                                show_detail.set(true);
                                                            }
                                                        },
                                                        {IconName::Eye.render()}
                                                    }
                                                    button {
                                                        class: "btn-icon btn-danger",
                                                        title: "Descargar certificado",
                                                        onclick: {
                                                            let email = r.customer_email.clone();
                                                            let name = r.customer_name.clone();
                                                            let plate = r.license_plate.clone();
                                                            move |e: Event<MouseData>| {
                                                                e.stop_propagation();
                                                                download_cert((email.clone(), name.clone(), plate.clone()));
                                                            }
                                                        },
                                                        disabled: *generating.read(),
                                                        if *generating.read() {
                                                            Spinner {}
                                                        } else {
                                                            {IconName::Download.render()}
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
            }

            if let Some(rid) = *detail_repair_id.read() {
                CertificateDetailModal {
                    show: *show_detail.read(),
                    repair_id: rid,
                    on_close: move |_| {
                        show_detail.set(false);
                        detail_repair_id.set(None);
                    },
                    on_download: {
                        let auth = auth;
                        let mut generating_set = generating;
                        let mut error_set = error;
                        let mut success_set = success_msg;
                        move |(email, name, plate): (Option<String>, Option<String>, Option<String>)| {
                            generating_set.set(true);
                            error_set.set(None);
                            success_set.set(None);
                            let client = auth.api_client();
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
                                                success_set.set(Some(format!("Certificado guardado en {}", path.display())));
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
                                }
                                generating_set.set(false);
                            });
                        }
                    },
                }
            }
        }
    }
}
