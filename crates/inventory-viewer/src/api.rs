use inventory_common::dto::{
    ApiResponse, CreateProductRequest, CreateRepairRequest, CreateSaleRequest,
    CreateSupplierRequest, LoginRequest, LoginResponse, PaginatedResponse, RegisterRequest,
};
use inventory_common::{Product, Repair, Sale, Supplier, User};
use serde::{Deserialize, Serialize};

const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

/// Error posible al llamar a la API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiError {
    Network(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    Validation(String),
    Server(String),
    Unknown(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(_) => write!(f, "{}", self.user_message()),
            ApiError::Unauthorized => write!(f, "La sesión ha expirado."),
            ApiError::Forbidden => write!(f, "No tienes permisos para realizar esta acción."),
            ApiError::NotFound(_) => write!(f, "El recurso solicitado no fue encontrado."),
            ApiError::Validation(_) => write!(f, "Revisa los datos ingresados."),
            ApiError::Server(_) => write!(f, "El servidor no pudo completar la operación."),
            ApiError::Unknown(_) => write!(f, "Ocurrió un error inesperado."),
        }
    }
}

impl ApiError {
    pub fn user_message(&self) -> &'static str {
        match self {
            ApiError::Network(_) => {
                "No se pudo conectar con el servidor. Verifica que esté encendido."
            }
            ApiError::Unauthorized => "La sesión ha expirado.",
            ApiError::Forbidden => "No tienes permisos para realizar esta acción.",
            ApiError::NotFound(_) => "El recurso solicitado no fue encontrado.",
            ApiError::Validation(_) => "Revisa los datos ingresados.",
            ApiError::Server(_) => "El servidor no pudo completar la operación.",
            ApiError::Unknown(_) => "Ocurrió un error inesperado.",
        }
    }
}

/// Cliente HTTP centralizado para comunicarse con el servidor.
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    token: Option<String>,
}

#[allow(dead_code)]
impl ApiClient {
    /// Crea un cliente nuevo.
    ///
    /// - `base_url`: URL base del servidor. Si es `None`, se usa el valor por defecto
    ///   de la configuración (`https://127.0.0.1:8443`).
    /// - `api_key`: clave compartida con el servidor para autenticar al viewer desktop.
    /// - `accept_invalid_certs`: si es `true`, permite conectar al certificado autofirmado
    ///   en desarrollo local.
    pub fn new(
        base_url: Option<String>,
        api_key: String,
        accept_invalid_certs: bool,
    ) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .build()
            .map_err(|e| ApiError::Network(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: base_url.unwrap_or_else(|| crate::config::config().server.base_url.clone()),
            api_key,
            token: None,
        })
    }

    /// Establece el token JWT usado en las siguientes peticiones.
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// POST /api/auth/login
    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse, ApiError> {
        let request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        self.post("/api/auth/login", &request).await
    }

    /// POST /api/auth/register
    /// Solo funciona si no existe ningún usuario (primer admin).
    pub async fn register(
        &self,
        workshop_name: &str,
        workshop_address: &str,
        workshop_city: &str,
        admin_name: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, ApiError> {
        let request = RegisterRequest {
            workshop_name: workshop_name.to_string(),
            workshop_address: workshop_address.to_string(),
            workshop_city: workshop_city.to_string(),
            admin_name: admin_name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };
        self.post("/api/auth/register", &request).await
    }

    /// GET /api/auth/status
    pub async fn status(&self) -> Result<User, ApiError> {
        self.get("/api/auth/status").await
    }

    /// GET /api/products
    pub async fn list_products(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Product>, ApiError> {
        self.get_with_query(
            "/api/products",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// GET /api/sales
    pub async fn list_sales(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Sale>, ApiError> {
        self.get_with_query(
            "/api/sales",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// GET /api/repairs
    pub async fn list_repairs(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Repair>, ApiError> {
        self.get_with_query(
            "/api/repairs",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// GET /api/suppliers
    pub async fn list_suppliers(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Supplier>, ApiError> {
        self.get_with_query(
            "/api/suppliers",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// POST /api/products
    pub async fn create_product(
        &self,
        request: &CreateProductRequest,
    ) -> Result<Product, ApiError> {
        self.post("/api/products", request).await
    }

    /// POST /api/sales
    pub async fn create_sale(&self, request: &CreateSaleRequest) -> Result<Sale, ApiError> {
        self.post("/api/sales", request).await
    }

    /// POST /api/repairs
    pub async fn create_repair(&self, request: &CreateRepairRequest) -> Result<Repair, ApiError> {
        self.post("/api/repairs", request).await
    }

    /// POST /api/suppliers
    pub async fn create_supplier(
        &self,
        request: &CreateSupplierRequest,
    ) -> Result<Supplier, ApiError> {
        self.post("/api/suppliers", request).await
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let mut request = self
            .client
            .get(self.url(path))
            .header("X-WorkshopManager-Key", &self.api_key);
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn get_with_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, ApiError> {
        let mut request = self
            .client
            .get(self.url(path))
            .query(query)
            .header("X-WorkshopManager-Key", &self.api_key);
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let mut request = self
            .client
            .post(self.url(path))
            .json(body)
            .header("X-WorkshopManager-Key", &self.api_key);
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<T, ApiError> {
        let response = response.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = response.status();
        let body_text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(format!("Failed to read response body: {}", e)))?;

        if status.is_success() {
            let parsed: ApiResponse<T> = serde_json::from_str(&body_text)
                .map_err(|e| ApiError::Unknown(format!("JSON parse error: {}", e)))?;
            parsed
                .data
                .ok_or_else(|| ApiError::Unknown("Empty response data".to_string()))
        } else {
            match status.as_u16() {
                401 => Err(ApiError::Unauthorized),
                403 => Err(ApiError::Forbidden),
                404 => Err(ApiError::NotFound(body_text)),
                422 => Err(ApiError::Validation(body_text)),
                500..=599 => Err(ApiError::Server(body_text)),
                _ => Err(ApiError::Unknown(body_text)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new(None, "test-key".to_string(), true);
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_client_default_url() {
        let client = ApiClient::new(None, "test-key".to_string(), true).unwrap();
        assert_eq!(client.base_url, crate::config::config().server.base_url);
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::Unauthorized;
        assert_eq!(err.to_string(), "Unauthorized");
    }
}
