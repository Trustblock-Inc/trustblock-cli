mod chains;
mod forward_request;
mod project;
mod trustblock_contracts;

pub use trustblock_contracts::TrustblockForwarder;

pub use forward_request::ForwardRequest;

pub use project::Project;

pub use chains::Chains;

use ethers::{ abi::Address, types::Bytes };

use serde::{ Deserialize, Serialize };

use crate::cmd::serialize_issues;

use validator::Validate;

use std::convert::From;

use clap::ValueEnum;

use strum::EnumIter;

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, ValueEnum, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Tag {
    Token,
    Finance,
    Collectibles,
    Gaming,
    Governance,
    Social,
    Other,
}

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, ValueEnum, Hash, Eq, PartialEq, Serialize)]
pub enum Status {
    #[serde(rename = "FIXED")]
    Fixed,
    #[serde(rename = "RISK_ACCEPTED")]
    RiskAccepted,
}

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, ValueEnum, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Default, Serialize, Validate)]
#[serde(rename_all = "UPPERCASE")]
pub struct SeverityCount {
    #[validate(range(max = 50))]
    pub low: u8,
    #[validate(range(max = 50))]
    pub medium: u8,
    #[validate(range(max = 50))]
    pub high: u8,
    #[validate(range(max = 50))]
    pub critical: u8,
}

impl SeverityCount {
    #[must_use]
    pub const fn new(low: u8, medium: u8, high: u8, critical: u8) -> Self {
        Self {
            low,
            medium,
            high,
            critical,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct IssueCount {
    #[serde(rename = "FIXED")]
    pub fixed: SeverityCount,
    #[serde(rename = "RISK_ACCEPTED")]
    pub risk_accepted: SeverityCount,
}

impl IssueCount {
    #[must_use]
    pub const fn new(fixed: SeverityCount, risk_accepted: SeverityCount) -> Self {
        Self {
            fixed,
            risk_accepted,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct Issue {
    pub status: Status,
    pub severity: Severity,
}

impl Issue {
    #[must_use]
    pub const fn new(status: Status, severity: Severity) -> Self {
        Self { status, severity }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub request: ForwardRequest,
    #[serde(rename = "domainSeparator")]
    pub domain_separator: Bytes,
    #[serde(rename = "requestTypeHash")]
    pub request_type_hash: Bytes,
    #[serde(rename = "suffixData")]
    pub suffix_data: Bytes,
    pub signature: Bytes,
    pub chain: String,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Contact {
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize)]
pub struct Audit {
    #[serde(skip_deserializing)]
    pub chains: Vec<Chains>,
    #[serde(serialize_with = "serialize_issues")]
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
    #[serde(skip_serializing)]
    pub project: Project,
    #[serde(skip_deserializing, rename = "projectId")]
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditContract {
    pub chain: Chains,
    #[serde(rename = "evmAddress")]
    pub evm_address: Address,
}

impl AuditContract {
    #[must_use]
    pub const fn new(chain: Chains, evm_address: Address) -> Self {
        Self { chain, evm_address }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub summary: String,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq, Eq)]
pub struct Links {
    #[validate(url)]
    pub twitter: Option<String>,
    #[validate(url)]
    pub github: Option<String>,
    #[validate(url)]
    pub website: Option<String>,
}