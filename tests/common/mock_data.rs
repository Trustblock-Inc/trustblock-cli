use serde::{ Deserialize, Serialize };
use validator::Validate;

use trustblock_cli::types::{ Chains, IssueCount, Tag, AuditContract, Description, Project };

#[derive(Debug, Clone, Serialize, Validate, Deserialize)]
pub struct MockAudit {
    #[serde(skip_deserializing)]
    pub chains: Vec<Chains>,
    pub issues: IssueCount,
    pub tags: Vec<Tag>,
    pub contracts: Vec<AuditContract>,
    pub description: Description,
    pub name: String,
    #[serde(rename = "reportHash", skip_deserializing)]
    pub report_hash: String,
    #[validate(url)]
    #[serde(rename = "reportFileUrl", skip_deserializing)]
    pub report_file_url: String,
    pub project: Project,
    #[serde(skip_deserializing, rename = "projectId")]
    pub project_id: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SQLIssueTempResult {
    pub status: String,
    pub severity: String,
    pub count: i64,
}