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
    Trial,
    Base,
    Reports,
    Advanced,
    Api,
}

impl LicenseTier {
    pub fn max_viewers(&self) -> u32 {
        match self {
            LicenseTier::Trial => 1,
            LicenseTier::Base => 2,
            LicenseTier::Reports => 5,
            LicenseTier::Advanced => 10,
            LicenseTier::Api => 999,
        }
    }

    pub fn max_transfers(&self) -> u32 {
        match self {
            LicenseTier::Trial => 0,
            LicenseTier::Base => 3,
            LicenseTier::Reports => 5,
            LicenseTier::Advanced => 10,
            LicenseTier::Api => 999,
        }
    }

    pub fn features(&self) -> Vec<Feature> {
        match self {
            LicenseTier::Trial => vec![
                Feature::Inventory,
                Feature::Sales,
                Feature::Repairs,
                Feature::Suppliers,
                Feature::Dashboard,
                Feature::Auth,
                Feature::AuditLog,
            ],
            LicenseTier::Base => vec![
                Feature::Inventory,
                Feature::Sales,
                Feature::Repairs,
                Feature::Suppliers,
                Feature::Dashboard,
                Feature::Auth,
                Feature::AuditLog,
                Feature::MultiViewer,
            ],
            LicenseTier::Reports => {
                let mut f = LicenseTier::Base.features();
                f.extend(vec![
                    Feature::PdfReports,
                    Feature::ClientHistory,
                    Feature::ExcelExport,
                ]);
                f
            }
            LicenseTier::Advanced => {
                let mut f = LicenseTier::Reports.features();
                f.extend(vec![
                    Feature::AdvancedAnalytics,
                    Feature::AutoBackup,
                    Feature::MultiWorkshop,
                ]);
                f
            }
            LicenseTier::Api => {
                let mut f = LicenseTier::Advanced.features();
                f.extend(vec![
                    Feature::RestApi,
                    Feature::Webhooks,
                    Feature::Integrations,
                ]);
                f
            }
        }
    }
}

impl std::fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseTier::Trial => write!(f, "Trial"),
            LicenseTier::Base => write!(f, "Base"),
            LicenseTier::Reports => write!(f, "Reports"),
            LicenseTier::Advanced => write!(f, "Advanced"),
            LicenseTier::Api => write!(f, "API"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub license_key: String,
    pub tier: LicenseTier,
    pub hardware_hash: String,
    pub max_viewers: u32,
    pub max_transfers: u32,
    pub transfer_count: u32,
    pub activated_at: chrono::DateTime<chrono::Utc>,
}

impl License {
    pub fn has_feature(&self, feature: &Feature) -> bool {
        self.tier.features().contains(feature)
    }

    pub fn is_valid(&self) -> bool {
        !self.license_key.is_empty() && !self.hardware_hash.is_empty()
    }

    pub fn can_transfer(&self) -> bool {
        self.transfer_count < self.max_transfers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_has_feature() {
        let license = License {
            license_key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            hardware_hash: "abc123".to_string(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
        };

        assert!(license.has_feature(&Feature::Inventory));
        assert!(license.has_feature(&Feature::Sales));
        assert!(!license.has_feature(&Feature::PdfReports));
    }

    #[test]
    fn test_license_is_valid() {
        let valid = License {
            license_key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            hardware_hash: "abc123".to_string(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
        };
        assert!(valid.is_valid());

        let invalid = License {
            license_key: String::new(),
            tier: LicenseTier::Base,
            hardware_hash: String::new(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_can_transfer() {
        let license = License {
            license_key: "TEST".to_string(),
            tier: LicenseTier::Base,
            hardware_hash: "abc".to_string(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 2,
            activated_at: chrono::Utc::now(),
        };
        assert!(license.can_transfer());

        let exhausted = License {
            transfer_count: 3,
            ..license
        };
        assert!(!exhausted.can_transfer());
    }

    #[test]
    fn test_tier_max_viewers() {
        assert_eq!(LicenseTier::Trial.max_viewers(), 1);
        assert_eq!(LicenseTier::Base.max_viewers(), 2);
        assert_eq!(LicenseTier::Reports.max_viewers(), 5);
        assert_eq!(LicenseTier::Advanced.max_viewers(), 10);
        assert_eq!(LicenseTier::Api.max_viewers(), 999);
    }
}
