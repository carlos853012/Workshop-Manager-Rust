use dioxus::prelude::*;
use inventory_common::Sale;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Sales() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let sales = use_signal(Vec::<Sale>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);

    let load_data = move || {
        let client = auth.api_client();
        let mut sales_set = sales;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.list_sales(current_page, 10).await {
                    Ok(response) => {
                        sales_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        error_set.set(Some("Sesión expirada".to_string()));
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
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
        load_data();
    });

    let columns: Vec<Column<Sale>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: |s| rsx! { span { "{s.customer_name.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "payment".to_string(),
            header: "Pago".to_string(),
            render: |s| rsx! { span { "{s.payment_method}" } },
        },
        Column {
            key: "total".to_string(),
            header: "Total".to_string(),
            render: |s| rsx! { span { "${s.total}" } },
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: |s| rsx! { span { class: "badge", "{s.status}" } },
        },
    ];

    let rows = sales.read().clone();

    rsx! {
        AppShell { title: "Ventas".to_string(), active_route: "/sales".to_string(),
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de ventas".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {},
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
        }
    }
}
