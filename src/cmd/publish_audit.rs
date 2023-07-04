use std::path::PathBuf;

use clap::{Parser, ValueHint};
use eyre::eyre;
use itertools::Itertools;
use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::{
    cmd::utils::{generate_pdf_from_url, upload_ipfs},
    constants::{AUDIT_ENDPOINT, TRUSTBLOCK_API_KEY_HEADER},
    types::{Audit, Chains, Project},
    utils::{apply_dotenv, parse_json, validate_links, validate_pdf},
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Parser)]
pub struct PublishAuditArgs {
    #[clap(
        short,
        long = "audit-data",
        help = "File path to JSON file with Audit data",
        value_name = "AUDIT_DATA_JSON_FILE",
        value_hint = ValueHint::FilePath,
        required(true)
    )]
    audit_file_path: PathBuf,

    #[clap(
        short,
        long = "report-pdf",
        help = "File path to audit report PDF file",
        value_name = "AUDIT_REPORT_PDF_FILE",
        value_hint = ValueHint::FilePath,
        value_parser = validate_pdf,
        required(true)
    )]
    report_pdf_file_path: Option<PathBuf>,

    #[clap(
        short = 'u',
        long = "report-url",
        help = "Url to audit report",
        value_name = "AUDIT_REPORT_URL",
        value_hint = ValueHint::Url,
        value_parser = validate_links,
        conflicts_with = "report_pdf_file_path",
        required(true)
    )]
    report_url: Option<String>,

    #[clap(short = 'k', long)]
    api_key: Option<String>,
}

impl PublishAuditArgs {
    #[allow(clippy::future_not_send)]
    pub async fn run(self) -> eyre::Result<()> {
        apply_dotenv()?;

        let audit_data = parse_json::<Audit>(&self.audit_file_path)?;

        let api_key = match self.api_key {
            Some(token) => token,
            None => std::env::var("API_KEY")?,
        };

        let project_id = audit_data
            .project
            .clone()
            .fetch_project_id(&api_key)
            .await?;

        let report_pdf_file_path = match self.report_pdf_file_path {
            Some(path) => path,
            None => {
                generate_pdf_from_url(self.report_url.expect("should not fail"), &api_key).await?
            }
        };

        let (report_hash, report_file_url) = upload_ipfs(report_pdf_file_path, &api_key).await?;

        let client = Client::new();

        let audit_endpoint =
            std::env::var("AUDIT_ENDPOINT").unwrap_or_else(|_| AUDIT_ENDPOINT.to_string());

        let chains = audit_data
            .contracts
            .iter()
            .map(|contract| contract.chain)
            .unique()
            .collect::<Vec<Chains>>();

        let project = Project {
            id: project_id,
            ..audit_data.clone().project
        };

        let audit_data_send = Audit {
            chains,
            report_hash: report_hash.clone(),
            report_file_url,
            project,
            ..audit_data
        };

        let response = client
            .post(audit_endpoint)
            .header(TRUSTBLOCK_API_KEY_HEADER, api_key)
            .json(&audit_data_send)
            .send()
            .await?;

        let status = response.status();

        let body = response.json::<Value>().await?;

        //TODO: create a separate error struct with "thiserror" crate
        match status {
            StatusCode::BAD_REQUEST => {
                let error = &body["error"];

                if error == "Report hash is not a unique value." {
                    println!("Audit already published to DB!\n");
                    return Ok(());
                }

                if error == "Project domain is not a unique value." {
                    println!("Project already exists on DB!\n");
                    return Ok(());
                }

                Err(eyre!(
                    "Could not publish to DB. Check validity of the audit data: {body} "
                ))
            }
            StatusCode::CREATED => {
                println!("Audit published successfully!\n");
                Ok(())
            }
            _ => Err(eyre!("Could not publish to DB. Response: {status}")),
        }
    }
}
