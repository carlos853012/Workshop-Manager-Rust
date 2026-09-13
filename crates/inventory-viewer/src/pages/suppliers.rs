use dioxus::prelude::*;
use inventory_common::Supplier;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::components::organisms::supplier_form_modal::SupplierFormModal;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;
use std::rc::Rc;

#[component]
pub fn Suppliers() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let suppliers = use_signal(Vec::<Supplier>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut show_modal = use_signal(|| false);
    let mut refresh = use_signal(|| 0);

    let load_data = move || {
        let client = auth.api_client();
        let mut suppliers_set = suppliers;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            match client {
                Some(client) => match client.list_suppliers(current_page, 10).await {
                    Ok(response) => {
                        suppliers_set.set(response.items);
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

    let columns: Vec<Column<Supplier>> = vec![
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: Rc::new(|s: &Supplier| rsx! { span { "{s.name}" } }),
        },
        Column {
            key: "contact".to_string(),
            header: "Contacto".to_string(),
            render: Rc::new(
                |s: &Supplier| rsx! { span { "{s.contact_person.as_deref().unwrap_or(\"-\")}" } },
            ),
        },
        Column {
            key: "email".to_string(),
            header: "Email".to_string(),
            render: Rc::new(
                |s: &Supplier| rsx! { span { "{s.email.as_deref().unwrap_or(\"-\")}" } },
            ),
        },
        Column {
            key: "phone".to_string(),
            header: "Teléfono".to_string(),
            render: Rc::new(
                |s: &Supplier| rsx! { span { "{s.phone.as_deref().unwrap_or(\"-\")}" } },
            ),
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: Rc::new(
                |s: &Supplier| rsx! { span { "{s.status}" } },
            ),
        },
        Column {
            key: "updated_at".to_string(),
            header: "Actualizado".to_string(),
            render: Rc::new(
                |s: &Supplier| rsx! { span { "{s.updated_at}" } },
            ),
        },
    ];

    let rows = suppliers.read().clone();

    rsx! {
        AppShell { title: "Proveedores".to_string(), active_route: Route::Suppliers {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de proveedores".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_modal.set(true),
                        "Nuevo proveedor"
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
            SupplierFormModal {
                show: *show_modal.read(),
                on_close: move |_| show_modal.set(false),
                on_saved: move |_| {
                    show_modal.set(false);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
            }
        }
    }
}
