use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Feature {
    // Base
    Inventory,
    Sales,
    Repairs,
    Suppliers,
    Dashboard,
    Auth,
    AuditLog,
    MultiViewer,
    // DLC: Reports
    PdfReports,
    ClientHistory,
    ExcelExport,
    // DLC: Advanced
    AdvancedAnalytics,
    AutoBackup,
    MultiWorkshop,
    // DLC: API
    RestApi,
    Webhooks,
    Integrations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseTier {
    Base,
    Reports,
    Advanced,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub tier: LicenseTier,
    pub features: Vec<Feature>,
    pub purchased_at: chrono::DateTime<chrono::Utc>,
    pub hardware_hash: Option<String>,
}

impl License {
    pub fn has_feature(&self, feature: &Feature) -> bool {
        self.features.contains(feature)
    }

    pub fn is_valid(&self) -> bool {
        !self.key.is_empty() && !self.features.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_has_feature() {
        let license = License {
            key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            features: vec![Feature::Inventory, Feature::Sales],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };

        assert!(license.has_feature(&Feature::Inventory));
        assert!(license.has_feature(&Feature::Sales));
        assert!(!license.has_feature(&Feature::PdfReports));
    }

    #[test]
    fn test_license_is_valid() {
        let valid = License {
            key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            features: vec![Feature::Inventory],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };
        assert!(valid.is_valid());

        let invalid = License {
            key: String::new(),
            tier: LicenseTier::Base,
            features: vec![],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };
        assert!(!invalid.is_valid());
    }
}
