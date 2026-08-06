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
