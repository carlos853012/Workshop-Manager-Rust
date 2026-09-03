use inventory_common::dto::{
    ApiResponse, CreateProductRequest, CreateRepairRequest, CreateSaleRequest,
    CreateSupplierRequest, LoginRequest, LoginResponse, PaginatedResponse,
};
use inventory_common::{Product, Repair, Sale, Supplier, User};
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://127.0.0.1:8443";
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
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Forbidden => write!(f, "Forbidden"),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ApiError::Server(msg) => write!(f, "Server error: {}", msg),
            ApiError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

/// Cliente HTTP centralizado para comunicarse con el servidor.
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

#[allow(dead_code)]
impl ApiClient {
    /// Crea un cliente nuevo. Si `accept_invalid_certs` es true, se permite conectar
    /// al certificado autofirmado del servidor en desarrollo local.
    pub fn new(base_url: Option<String>, accept_invalid_certs: bool) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .build()
            .map_err(|e| ApiError::Network(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
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
    pub async fn register(&self, email: &str, password: &str) -> Result<LoginResponse, ApiError> {
        let request = LoginRequest {
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
        let mut request = self.client.get(self.url(path));
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
        let mut request = self.client.get(self.url(path)).query(query);
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
        let mut request = self.client.post(self.url(path)).json(body);
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
        let client = ApiClient::new(None, true);
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_client_default_url() {
        let client = ApiClient::new(None, true).unwrap();
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::Unauthorized;
        assert_eq!(err.to_string(), "Unauthorized");
    }
}
