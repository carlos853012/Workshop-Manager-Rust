use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

pub fn default_page() -> i32 {
    1
}

pub fn default_per_page() -> i32 {
    20
}

impl PaginationParams {
    pub fn offset(&self) -> i32 {
        (self.page - 1) * self.per_page
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.page < 1 {
            return Err("page must be >= 1".to_string());
        }
        if self.per_page < 1 || self.per_page > 100 {
            return Err("per_page must be between 1 and 100".to_string());
        }
        Ok(())
    }
}
