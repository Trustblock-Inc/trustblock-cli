use crate::{
    constants::{ AUDIT_ENDPOINT, TRUSTBLOCK_API_KEY_HEADER },
    types::{ Audit, Chains },
    utils::apply_dotenv,
};

use reqwest::{ Client, StatusCode };

use itertools::Itertools;

use serde_json::Value;

use eyre::eyre;

pub async fn publish_audit_db(
    audit_data: Audit,
    project_id: String,
    report_hash: String,
    report_file_url: String,
    api_key: &str
) -> eyre::Result<Audit> {
    let client = Client::new();

    apply_dotenv()?;

    let audit_endpoint = std::env
        ::var("AUDIT_ENDPOINT")
        .unwrap_or_else(|_| AUDIT_ENDPOINT.to_string());

    let chains = audit_data.contracts
        .iter()
        .map(|contract| contract.chain)
        .unique()
        .collect::<Vec<Chains>>();

    let audit_data_send = Audit {
        chains,
        report_hash: report_hash.clone(),
        report_file_url,
        project_id,
        ..audit_data.clone()
    };

    let response = client
        .post(audit_endpoint)
        .header(TRUSTBLOCK_API_KEY_HEADER, api_key)
        .json(&audit_data_send)
        .send().await?;

    let status = response.status();

    let body = response.json::<Value>().await?;

    match status {
        StatusCode::BAD_REQUEST => {
            if body["error"] == "Report hash is not a unique value." {
                println!("Audit already published to DB!\n");
                return Ok(audit_data_send);
            }
            Err(eyre!("Could not publish to DB. Check validity of the audit data: {body} "))
        }
        StatusCode::CREATED => {
            println!("Audit published successfully to DB!\n");
            Ok(audit_data_send)
        }
        _ => Err(eyre!("Could not publish to DB. Response: {status}")),
    }
}