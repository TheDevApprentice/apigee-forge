use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApigeeRole {
    #[serde(rename = "apigee.admin")]
    Admin,
    #[serde(rename = "apigee.readOnlyAdmin")]
    ReadOnlyAdmin,
    #[serde(rename = "apigee.developerAdmin")]
    DeveloperAdmin,
    #[serde(rename = "apigee.analyticsViewer")]
    AnalyticsViewer,
    #[serde(rename = "apigee.deployer")]
    Deployer,
    #[serde(rename = "apigee.portalAdmin")]
    PortalAdmin,
}

impl ApigeeRole {
    pub fn from_iam_name(role: &str) -> Option<Self> {
        match role.strip_prefix("roles/").unwrap_or(role) {
            "apigee.admin" => Some(Self::Admin),
            "apigee.readOnlyAdmin" => Some(Self::ReadOnlyAdmin),
            "apigee.developerAdmin" => Some(Self::DeveloperAdmin),
            "apigee.analyticsViewer" => Some(Self::AnalyticsViewer),
            "apigee.deployer" => Some(Self::Deployer),
            "apigee.portalAdmin" => Some(Self::PortalAdmin),
            _ => None,
        }
    }
}
