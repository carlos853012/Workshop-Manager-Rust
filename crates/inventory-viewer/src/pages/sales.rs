use dioxus::prelude::*;
use dioxus_router::prelude::*;
use inventory_common::money::format_clp;
use crate::i18n;
use inventory_common::Sale;
use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::components::organisms::sale_detail_modal::SaleDetailModal;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;
use std::rc::Rc;

#[component]
pub fn Sales() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let sales = use_signal(Vec::<Sale>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let refresh = use_signal(|| 0);
    let mut show_detail = use_signal(|| false);
    let mut detail_sale_id = use_signal(|| None::<uuid::Uuid>);

    let load_data = move || {
        let client = auth.api_client();
        let mut sales_set = sales;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            match client {
                Some(client) => match client.list_sales(current_page, 10).await {
                    Ok(response) => {
                        sales_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            loading_set.set(false);
        });
    };

    use_effect(move || {
        let _ = *refresh.read();
        load_data();
    });

    let columns: Vec<Column<Sale>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: Rc::new(
                |s: &Sale| rsx! { span { "{s.customer_name.as_deref().unwrap_or(\"-\")}" } },
            ),
        },
        Column {
            key: "payment".to_string(),
            header: "Pago".to_string(),
            render: Rc::new(|s: &Sale| rsx! { span { "{i18n::translate_payment(&s.payment_method)}" } }),
        },
        Column {
            key: "total".to_string(),
            header: "Total".to_string(),
            render: Rc::new(
                |s: &Sale| rsx! { span { class: "text-right", "{format_clp(s.total)}" } },
            ),
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: Rc::new(|s: &Sale| rsx! { span { "{i18n::translate_sale_status(&s.status)}" } }),
        },
        Column {
            key: "actions".to_string(),
            header: "".to_string(),
            render: Rc::new({
                let mut sale_id = detail_sale_id;
                let mut show_detail = show_detail;
                move |s: &Sale| {
                    let sid = s.id;
                    rsx! {
                        div { class: "table-actions",
                            button {
                                class: "btn-icon btn-edit",
                                title: "Ver detalle",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    sale_id.set(Some(sid));
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

    let rows = sales.read().clone();

    rsx! {
        AppShell { title: "Ventas".to_string(), active_route: Route::Sales {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Historial de ventas".to_string(),
                header_action: rsx! {
                    Link {
                        to: Route::Pos {},
                        class: "btn btn-primary",
                        "Nueva venta"
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
            if let Some(sid) = *detail_sale_id.read() {
                SaleDetailModal {
                    key: "{sid}",
                    show: *show_detail.read(),
                    sale_id: sid,
                    on_close: move |_| {
                        show_detail.set(false);
                        detail_sale_id.set(None);
                    },
                }
            }
        }
    }
}
