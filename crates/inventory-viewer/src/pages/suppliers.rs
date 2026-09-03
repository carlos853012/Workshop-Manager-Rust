use dioxus::prelude::*;
use inventory_common::dto::CreateSupplierRequest;
use inventory_common::Supplier;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Suppliers() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
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

        spawn(async move {
            match client {
                Some(client) => match client.list_suppliers(current_page, 10).await {
                    Ok(response) => {
                        suppliers_set.set(response.items);
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
        let _ = *refresh.read();
        load_data();
    });

    let columns: Vec<Column<Supplier>> = vec![
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: |s| rsx! { span { "{s.name}" } },
        },
        Column {
            key: "contact".to_string(),
            header: "Contacto".to_string(),
            render: |s| rsx! { span { "{s.contact_person.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "email".to_string(),
            header: "Email".to_string(),
            render: |s| rsx! { span { "{s.email.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "phone".to_string(),
            header: "Teléfono".to_string(),
            render: |s| rsx! { span { "{s.phone.as_deref().unwrap_or(\"-\")}" } },
        },
    ];

    let rows = suppliers.read().clone();

    rsx! {
        AppShell { title: "Proveedores".to_string(), active_route: "/suppliers".to_string(),
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

#[component]
fn SupplierFormModal(show: bool, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut name = use_signal(|| "".to_string());
    let mut contact_person = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut phone = use_signal(|| "".to_string());
    let mut address = use_signal(|| "".to_string());
    let mut tax_id = use_signal(|| "".to_string());
    let mut payment_terms = use_signal(|| "".to_string());
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let on_submit = move |_| {
        let name_value = name.read().trim().to_string();
        if name_value.is_empty() {
            form_error.set(Some("El nombre es obligatorio".to_string()));
            return;
        }

        let request = CreateSupplierRequest {
            name: name_value,
            contact_person: Some(contact_person.read().clone()).filter(|s| !s.is_empty()),
            email: Some(email.read().clone()).filter(|s| !s.is_empty()),
            phone: Some(phone.read().clone()).filter(|s| !s.is_empty()),
            address: Some(address.read().clone()).filter(|s| !s.is_empty()),
            tax_id: Some(tax_id.read().clone()).filter(|s| !s.is_empty()),
            payment_terms: Some(payment_terms.read().clone()).filter(|s| !s.is_empty()),
        };

        let client = auth.api_client();
        let mut saving_set = saving;
        let mut error_set = form_error;
        let on_saved_clone = on_saved;

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.create_supplier(&request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            saving_set.set(false);
        });
    };

    rsx! {
        Modal {
            show: show,
            title: "Nuevo proveedor".to_string(),
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    loading: *saving.read(),
                    onclick: on_submit,
                    "Guardar"
                }
            },
            if let Some(err) = form_error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            div { class: "form-row",
                Input {
                    label: Some("Nombre *".to_string()),
                    value: name.read().clone(),
                    oninput: move |evt: FormEvent| name.set(evt.value().clone()),
                }
                Input {
                    label: Some("Contacto".to_string()),
                    value: contact_person.read().clone(),
                    oninput: move |evt: FormEvent| contact_person.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Email".to_string()),
                    r#type: "email".to_string(),
                    value: email.read().clone(),
                    oninput: move |evt: FormEvent| email.set(evt.value().clone()),
                }
                Input {
                    label: Some("Teléfono".to_string()),
                    value: phone.read().clone(),
                    oninput: move |evt: FormEvent| phone.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Dirección".to_string()),
                    value: address.read().clone(),
                    oninput: move |evt: FormEvent| address.set(evt.value().clone()),
                }
                Input {
                    label: Some("CUIT/RUC".to_string()),
                    value: tax_id.read().clone(),
                    oninput: move |evt: FormEvent| tax_id.set(evt.value().clone()),
                }
            }
            div { class: "mt-md" }
            Input {
                label: Some("Términos de pago".to_string()),
                value: payment_terms.read().clone(),
                oninput: move |evt: FormEvent| payment_terms.set(evt.value().clone()),
            }
        }
    }
}
