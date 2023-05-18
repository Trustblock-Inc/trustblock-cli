mod publish_db;
mod publish_sc;

use crate::types::{ Audit };

use publish_db::publish_audit_db;

use publish_sc::publish_audit_sc;

use crate::{
    cmd::utils::{ upload_ipfs, generate_pdf_from_url },
    utils::{ apply_dotenv, parse_json, validate_pdf, validate_links },
};

use std::{ path::PathBuf, println };

use clap::{ Parser, ValueHint };

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

    #[clap(short = 'p', long)]
    private_key: Option<String>,

    #[clap(short = 's', long, help = "Publish an audit to SC", default_value = "false")]
    publish_sc: bool,
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

        let project_id = audit_data.project.clone().fetch_project_id(&api_key).await?;

        let report_pdf_file_path = match self.report_pdf_file_path {
            Some(path) => path,
            None => {
                generate_pdf_from_url(self.report_url.expect("should not fail"), &api_key).await?
            }
        };

        let (report_hash, report_file_url) = upload_ipfs(report_pdf_file_path, &api_key).await?;

        let audit_data = publish_audit_db(
            audit_data,
            project_id,
            report_hash.clone(),
            report_file_url,
            &api_key
        ).await?;

        if self.publish_sc {
            publish_audit_sc(
                &audit_data,
                &audit_data.project.name,
                report_hash,
                self.private_key,
                &api_key
            ).await?;
        }

        println!("Audit published successfully");

        Ok(())
    }
}