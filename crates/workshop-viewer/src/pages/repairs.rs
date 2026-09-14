use dioxus::prelude::*;
use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::components::organisms::repair_detail_modal::RepairDetailModal;
use crate::components::organisms::repair_form_modal::RepairFormModal;
use crate::icons::IconName;
use crate::i18n;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;
use std::rc::Rc;

#[component]
pub fn Repairs() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let repairs = use_signal(Vec::<workshop_common::Repair>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut show_create_modal = use_signal(|| false);
    let mut refresh = use_signal(|| 0);
    let mut show_detail_modal = use_signal(|| false);
    let mut detail_repair_id = use_signal(|| None::<uuid::Uuid>);

    let load_data = move || {
        let client = auth.api_client();
        let mut repairs_set = repairs;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            if let Some(client) = client {
                match client.list_repairs(current_page, 10).await {
                    Ok(response) => {
                        repairs_set.set(response.items);
                        total_set.set(response.total);
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
        let _ = *refresh.read();
        load_data();
    });

    let columns: Vec<Column<workshop_common::Repair>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: Rc::new(|r: &workshop_common::Repair| {
                rsx! { span { "{r.customer_name.as_deref().unwrap_or(\"-\")}" } }
            }),
        },
        Column {
            key: "vehicle".to_string(),
            header: "Vehículo".to_string(),
            render: Rc::new(|r: &workshop_common::Repair| {
                rsx! { span { "{r.vehicle.as_deref().unwrap_or(\"-\")}" } }
            }),
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: Rc::new(|r: &workshop_common::Repair| {
                rsx! {
                    span { "{i18n::translate_repair_status(&r.status)}" }
                }
            }),
        },
        Column {
            key: "priority".to_string(),
            header: "Prioridad".to_string(),
            render: Rc::new(|r: &workshop_common::Repair| {
                rsx! { span { "{i18n::translate_priority(&r.priority)}" } }
            }),
        },
        Column {
            key: "actions".to_string(),
            header: "".to_string(),
            render: Rc::new({
                let mut detail_id = detail_repair_id;
                let mut show_detail = show_detail_modal;
                move |r: &workshop_common::Repair| {
                    let rid = r.id;
                    rsx! {
                        div { class: "table-actions",
                            button {
                                class: "btn-icon btn-edit",
                                title: "Ver detalle",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    detail_id.set(Some(rid));
                                    show_detail.set(true);
                                },
                                {IconName::Eye.render()}
                            }
                        }
                    }
                }
            }),
        },
    ];

    let rows = repairs.read().clone();

    rsx! {
        AppShell { title: "Reparaciones".to_string(), active_route: Route::Repairs {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de reparaciones".to_string(),
                header_action: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_create_modal.set(true),
                        "Nueva reparación"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    DataTable {
                        columns: columns.clone(),
                        rows: rows.clone(),
                        page: *page.read(),
                        total: *total.read(),
                        on_page_change: move |new_page: i32| page.set(new_page),
                    }
                }
            }
            RepairFormModal {
                show: *show_create_modal.read(),
                on_close: move |_| show_create_modal.set(false),
                on_saved: move |_| {
                    show_create_modal.set(false);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
            }
            if let Some(rid) = *detail_repair_id.read() {
                RepairDetailModal {
                    key: "{rid}",
                    show: *show_detail_modal.read(),
                    repair_id: rid,
                    on_close: move |_| {
                        show_detail_modal.set(false);
                        detail_repair_id.set(None);
                    },
                    on_saved: move |_| {
                        let current = *refresh.read();
                        refresh.set(current + 1);
                    },
                }
            }
        }
    }
}
