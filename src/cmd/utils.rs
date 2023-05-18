use crate::{
    constants::{ TRUSTBLOCK_API_KEY_HEADER, WEB3_STORAGE_API_ENDPOINT, WEB3_STORAGE_ENDPOINT },
    types::{ AuditContract, Chains, Issue, IssueCount, Severity, Status },
    utils::apply_dotenv,
};

use std::{ future::Future, path::PathBuf, sync::{ Arc, Mutex } };

use ethers::abi::{ FixedBytes, Function, Token };

use serde::ser::{ Serialize, Serializer };

use eyre::{ eyre, ContextCompat };

use serde_json::Value;

use reqwest::Client;

use w3s::helper;

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> eyre::Result<()>;
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
    rt.block_on(future)
}

#[allow(clippy::future_not_send)]
pub async fn upload_ipfs(
    report_file_path: PathBuf,
    auth_token: &String
) -> eyre::Result<(String, String)> {
    apply_dotenv()?;

    let client = Client::new();

    let web3_storage_endpoint = std::env
        ::var("WEB3_STORAGE_API_ENDPOINT")
        .unwrap_or_else(|_| WEB3_STORAGE_API_ENDPOINT.to_string());

    let response = client
        .post(web3_storage_endpoint)
        .header(TRUSTBLOCK_API_KEY_HEADER, auth_token)
        .send().await?;

    if !response.status().is_success() {
        return Err(eyre!("Could not upload to report. Try again"));
    }

    let api_response_data = response.json::<Value>().await?;

    let api_key = &api_response_data["apiKey"].to_string().trim().replace('\"', "");

    if api_key.is_empty() {
        return Err(eyre!("Could not upload to report. Try again"));
    }

    let results = helper::upload(
        report_file_path.to_str().wrap_err("Invalid File Path")?,
        api_key.as_str(),
        2,
        Some(
            Arc::new(
                Mutex::new(|name, _, pos, total| {
                    if pos != 0 {
                        if pos == total {
                            println!("[+] Uploading is done\n");
                        } else {
                            let percentage = (pos * 100) / total;
                            println!("[+] Uploading {name}. Finished %{percentage:}");
                        }
                    }
                })
            )
        ),
        None,
        None,
        None
    ).await?;

    let cid = results[0].to_string();

    let report_url = format!("https://{cid}{WEB3_STORAGE_ENDPOINT}");

    Ok((cid, report_url))
}

pub fn get_message_data(
    publish_audit_function: &Function,
    chain: &Chains,
    contracts: &[AuditContract],
    project_name: [u8; 28],
    report_hash: String,
    issue_bytes: [u8; 4]
) -> eyre::Result<Vec<u8>> {
    let contract_tokens = contracts
        .iter()
        .filter(|contract| contract.chain.eq(chain))
        .map(|contract| Token::Address(contract.evm_address))
        .collect::<Vec<Token>>();

    let fixed_bytes: FixedBytes = issue_bytes.into();

    let encoded = publish_audit_function.encode_input(
        &[
            Token::Array(contract_tokens),
            Token::String(report_hash),
            Token::FixedBytes(project_name.into()),
            Token::FixedBytes(fixed_bytes),
        ]
    )?;

    Ok(encoded)
}

pub fn serialize_issues<S>(issues: &IssueCount, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    let transformed_issues = get_transformed_issues(issues);
    transformed_issues.serialize(serializer)
}

#[must_use]
pub fn get_transformed_issues(issues: &IssueCount) -> Vec<Issue> {
    let mut result: Vec<Issue> = Vec::new();

    for (status, severity_count) in [
        (Status::Fixed, issues.fixed),
        (Status::RiskAccepted, issues.risk_accepted),
    ] {
        for (severity, count) in [
            (Severity::Low, severity_count.low),
            (Severity::Medium, severity_count.medium),
            (Severity::High, severity_count.high),
            (Severity::Critical, severity_count.critical),
        ] {
            for _ in 0..count {
                result.push(Issue { status, severity });
            }
        }
    }

    result
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn get_issue_bytes(issues: IssueCount) -> [u8; 4] {
    let risk_accepted = issues.risk_accepted;

    (
        (u32::from(risk_accepted.critical) << 24) |
        (u32::from(risk_accepted.high) << 16) |
        (u32::from(risk_accepted.medium) << 8) |
        u32::from(risk_accepted.low)
    ).to_be_bytes()
}