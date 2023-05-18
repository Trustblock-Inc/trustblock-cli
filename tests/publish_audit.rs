mod common;

use common::{
    constants::FIXTURES_DIR,
    generate_random_data,
    utils::{ clean_temp_file, connect_db },
    mock_data::SQLIssueTempResult,
};

use ethers::contract::abigen;
use ethers_core::types::Address;
use sqlx::{ mysql::MySqlRow, Row };
use trustblock_cli::types::{ AuditContract, Chains, Links, SeverityCount, IssueCount };

use predicates::prelude::*;

use assert_cmd::Command;

abigen!(TrustblockCore, "tests/test-data/tb_core_abi.json");

#[tokio::test]
async fn test_publish_audit_report_and_publish_audit_project_exists() -> eyre::Result<()> {
    let (pdf_file, audit_file, audit) = generate_random_data()?;

    let publish_db_fixture = format!("{}{}", FIXTURES_DIR, "publish_db.stdout");

    let publish_db_report_exists_project_exists = format!(
        "{}{}",
        FIXTURES_DIR,
        "publish_db_report_exists_project_exists.stdout"
    );

    let pool = connect_db().await?;

    let project_links = audit.project.links;

    let pattern = std::fs::read_to_string(publish_db_fixture)?;

    // Project Created & Audit published
    Command::cargo_bin("trustblock")?
        .arg("publish-audit")
        .arg("-a")
        .arg(&audit_file)
        .arg("-r")
        .arg(&pdf_file)
        .assert()
        .success()
        .stdout(predicate::str::is_match(pattern)?);

    let pattern = std::fs::read_to_string(publish_db_report_exists_project_exists)?;

    // Project & Report exists. No project creation and audit publishing
    Command::cargo_bin("trustblock")?
        .arg("publish-audit")
        .arg("-a")
        .arg(&audit_file)
        .arg("-r")
        .arg(&pdf_file)
        .assert()
        .success()
        .stdout(predicate::str::is_match(pattern)?);

    clean_temp_file(pdf_file)?;
    clean_temp_file(audit_file)?;

    let fetched_project_names = sqlx
        ::query("SELECT name FROM Project WHERE name = ?")
        .bind(&audit.project.name)
        .fetch_all(&pool).await?;

    assert!(fetched_project_names.len() == 1, "Only one project should be added");

    let fetched_project_name = fetched_project_names[0].get::<String, _>("name");

    assert!(
        audit.project.name == fetched_project_name,
        "Project name does not equal queried project name"
    );

    // prettier-ignore
    let fetched_links = sqlx
        ::query_as!(
            Links,
            "SELECT website, twitter, github FROM Links WHERE website = ?",
            project_links.website.as_ref().unwrap()
        )
        .fetch_all(&pool).await?;

    assert!(fetched_links.len() == 1, "Only one link row should be added");

    assert!(project_links == fetched_links[0], "Project links do not equal queried project links");

    let fetched_audit_data = sqlx
        ::query("SELECT id, name FROM Audit WHERE name = ?")
        .bind(&audit.name)
        .fetch_all(&pool).await?;

    assert!(fetched_audit_data.len() == 1, "Only one audit should not be added");

    let fetched_audit_name = fetched_audit_data[0].get::<String, _>("name");

    assert!(audit.name == fetched_audit_name, "Audit name does not equal queried Audit name");

    let fetched_contract_data = sqlx
        ::query("SELECT id, evmAddress, chain FROM Contract")
        .map(|row: MySqlRow| {
            let chain = row.get::<&str, _>("chain").parse::<Chains>().unwrap();
            let evm_address = row.get::<&str, _>("evmAddress").parse::<Address>().unwrap();

            let id = row.get::<String, _>("id");

            (id, AuditContract::new(chain, evm_address))
        })
        .fetch_all(&pool).await?;

    let fetched_contracts = fetched_contract_data
        .iter()
        .map(|contract| contract.1.clone())
        .collect::<Vec<AuditContract>>();

    let is_contract_added = audit.contracts
        .iter()
        .all(|contract| fetched_contracts.contains(contract));

    assert!(is_contract_added, "Missing contracts");

    let fetched_audit_to_contract = sqlx
        ::query("SELECT A, B FROM _AuditToContract")
        .map(|row: MySqlRow| {
            let audit = row.get::<String, _>("A");
            let contract = row.get::<String, _>("B");

            (audit, contract)
        })
        .fetch_all(&pool).await?;

    let is_mapping_exists = fetched_audit_data
        .iter()
        .zip(fetched_contract_data)
        .all(|(audit, contract)| {
            fetched_audit_to_contract.contains(&(audit.get::<String, _>("id"), contract.0))
        });

    assert!(is_mapping_exists, "Missing mapping between audit to contract");

    let fetched_issue_count = sqlx
        ::query_as::<_, SQLIssueTempResult>(
            r#"
        SELECT status, severity, COUNT(*) as count
        FROM Issue
        GROUP BY status, severity
        ORDER BY status, severity
        "#
        )
        .fetch_all(&pool).await?;

    let (fixed, risk_accepted) = fetched_issue_count
        .into_iter()
        .fold(
            (SeverityCount::default(), SeverityCount::default()),
            |(mut fixed, mut risk_accepted), result| {
                let count = result.count;

                match (result.status.as_str(), result.severity.as_str()) {
                    ("FIXED", "LOW") => {
                        fixed.low = count as u8;
                    }
                    ("FIXED", "MEDIUM") => {
                        fixed.medium = count as u8;
                    }
                    ("FIXED", "HIGH") => {
                        fixed.high = count as u8;
                    }
                    ("FIXED", "CRITICAL") => {
                        fixed.critical = count as u8;
                    }
                    ("RISK_ACCEPTED", "LOW") => {
                        risk_accepted.low = count as u8;
                    }
                    ("RISK_ACCEPTED", "MEDIUM") => {
                        risk_accepted.medium = count as u8;
                    }
                    ("RISK_ACCEPTED", "HIGH") => {
                        risk_accepted.high = count as u8;
                    }
                    ("RISK_ACCEPTED", "CRITICAL") => {
                        risk_accepted.critical = count as u8;
                    }
                    _ => (),
                }

                (fixed, risk_accepted)
            }
        );

    let fetched_issue_count = IssueCount::new(fixed, risk_accepted);

    assert!(audit.issues == fetched_issue_count, "Issue counts are not equal");

    Ok(())
}