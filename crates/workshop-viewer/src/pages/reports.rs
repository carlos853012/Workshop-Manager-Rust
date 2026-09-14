use dioxus::prelude::*;
use workshop_common::dto::{ClientHistoryResponse, ClientReport};
use workshop_common::money::format_clp;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Reports() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let clients = use_signal(Vec::<ClientReport>::new);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut selected_client = use_signal(|| None::<ClientHistoryResponse>);

    let load_clients = move || {
        let client = auth.api_client();
        let mut clients_set = clients;
        let mut loading_set = loading;
        let mut error_set = error;

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            if let Some(api) = client {
                match api.list_clients().await {
                    Ok(data) => {
                        clients_set.set(data);
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
            loading_set.set(false);
        });
    };

    use_effect(move || {
        load_clients();
    });

    let rows = clients.read().clone();

    if let Some(history) = selected_client.read().as_ref() {
        return rsx! {
            AppShell { title: "Historial de cliente".to_string(), active_route: Route::Reports {},
                div { class: "flex items-center gap-md mb-lg",
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| selected_client.set(None),
                        {IconName::ChevronLeft.render()}
                        " Volver"
                    }
                    h2 { class: "text-lg font-semibold", "{history.email}" }
                    if let Some(ref name) = history.name {
                        span { class: "text-muted ml-sm", "({name})" }
                    }
                }
                div { class: "grid grid-3 mb-lg",
                    Card {
                        p { class: "text-muted text-sm", "Total gastado" }
                        p { class: "text-xl font-semibold", "{format_clp(history.total_spent)}" }
                    }
                    Card {
                        p { class: "text-muted text-sm", "Compras" }
                        p { class: "text-xl font-semibold", "{history.sales.len()}" }
                    }
                    Card {
                        p { class: "text-muted text-sm", "Reparaciones" }
                        p { class: "text-xl font-semibold", "{history.repairs.len()}" }
                    }
                }
                Card { title: "Ventas".to_string(),
                    if history.sales.is_empty() {
                        div { class: "empty-state", "Sin ventas registradas" }
                    } else {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "Fecha" }
                                    th { "Pago" }
                                    th { class: "text-right", "Total" }
                                }
                            }
                            tbody {
                                for sale in history.sales.iter() {
                                    tr {
                                        td { class: "text-muted", "{sale.created_at.format(\"%d-%m-%Y\")}" }
                                        td { "{sale.payment_method}" }
                                        td { class: "text-right", "{format_clp(sale.total)}" }
                                    }
                                }
                            }
                        }
                    }
                }
                Card { title: "Reparaciones".to_string(),
                    if history.repairs.is_empty() {
                        div { class: "empty-state", "Sin reparaciones registradas" }
                    } else {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "Fecha" }
                                    th { "Estado" }
                                    th { "Descripción" }
                                    th { class: "text-right", "Total" }
                                }
                            }
                            tbody {
                                for repair in history.repairs.iter() {
                                    tr {
                                        td { class: "text-muted", "{repair.created_at.format(\"%d-%m-%Y\")}" }
                                        td { "{repair.status}" }
                                        td { class: "text-muted", "{repair.description.as_deref().unwrap_or(\"-\")}" }
                                        td { class: "text-right",
                                            if let Some(total) = repair.total {
                                                span { "{format_clp(total)}" }
                                            } else {
                                                span { class: "text-muted", "-" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        AppShell { title: "Reportes".to_string(), active_route: Route::Reports {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Clientes".to_string(),
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    div { class: "data-table-wrapper",
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "Email" }
                                    th { "Nombre" }
                                    th { "Compras" }
                                    th { class: "text-right", "Total gastado" }
                                    th { "Última compra" }
                                }
                            }
                            tbody {
                                if rows.is_empty() {
                                    tr {
                                        td { colspan: "5", div { class: "empty-state", "No hay clientes" } }
                                    }
                                } else {
                                    for c in rows.iter() {
                                        tr {
                                            class: "clickable-row",
                                            onclick: {
                                                let selected_client = selected_client;
                                                let email = c.customer_email.clone();
                                                move |_| {
                                                    let api = match auth.api_client() {
                                                        Some(c) => c,
                                                        None => return,
                                                    };
                                                    let email = email.clone();
                                                    let mut selected_client = selected_client;
                                                    spawn(async move {
                                                        if let Ok(history) = api.get_client_history(&email).await {
                                                            selected_client.set(Some(history));
                                                        }
                                                    });
                                                }
                                            },
                                            td { "{c.customer_email}" }
                                            td { class: "text-muted", "{c.customer_name.as_deref().unwrap_or(\"-\")}" }
                                            td { "{c.total_purchases}" }
                                            td { class: "text-right", "{format_clp(c.total_spent)}" }
                                            td { class: "text-muted",
                                                "{c.last_purchase.map(|d| d.format(\"%d-%m-%Y\").to_string()).unwrap_or_else(|| \"-\".to_string())}"
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
