mod common;

use common::{
    constants::FIXTURES_DIR,
    generate_random_data,
    utils::generate_random_audit,
    utils::{ clean_temp_file, connect_db },
    mock_data::SQLIssueTempResult,
};

use ethers_core::types::Address;
use sqlx::{ mysql::MySqlRow, Row };
use trustblock_cli::types::{ AuditContract, Chains, SeverityCount, IssueCount, Links };

use predicates::prelude::*;

use assert_cmd::Command;

#[tokio::test]
async fn test_publish_audit_db() -> eyre::Result<()> {
    let (pdf_file, audit_file, audit) = generate_random_data(Some(1))?;

    let publish_db_fixture = format!("{}{}", FIXTURES_DIR, "publish_db.stdout");

    let publish_db_report_exists_project_exists_fixture = format!(
        "{}{}",
        FIXTURES_DIR,
        "publish_db_report_exists_project_exists.stdout"
    );

    let args_report_and_url_together_error_fixture = format!(
        "{}{}",
        FIXTURES_DIR,
        "args_report_and_url_together_error.stdout"
    );

    let web_report_url = "https://trustblock.run/";

    let pool = connect_db().await?;

    let project_links = audit.project.links;

    let pattern = std::fs::read_to_string(&publish_db_fixture)?;

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

    let pattern = std::fs::read_to_string(publish_db_report_exists_project_exists_fixture)?;

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

    let pattern = std::fs::read_to_string(args_report_and_url_together_error_fixture)?;

    // Url and Report args cannot be used together
    Command::cargo_bin("trustblock")?
        .arg("publish-audit")
        .arg("-a")
        .arg(&audit_file)
        .arg("-r")
        .arg(&pdf_file)
        .arg("-u")
        .arg(web_report_url)
        .assert()
        .failure()
        .stderr(predicate::str::is_match(pattern)?);

    clean_temp_file(audit_file)?;
    clean_temp_file(pdf_file)?;

    let (audit_file, web_audit) = generate_random_audit(None)?;

    let pattern = std::fs::read_to_string(&publish_db_fixture)?;

    // Publishing web based report
    Command::cargo_bin("trustblock")?
        .arg("publish-audit")
        .arg("-a")
        .arg(&audit_file)
        .arg("-u")
        .arg(web_report_url)
        .assert()
        .success()
        .stdout(predicate::str::is_match(pattern)?);

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

    let fetched_audit_data_web = sqlx
        ::query("SELECT id, name FROM Audit WHERE name = ?")
        .bind(&web_audit.name)
        .fetch_all(&pool).await?;

    assert!(fetched_audit_data.len() == 1, "Only one audit should not be added");

    assert!(fetched_audit_data_web.len() == 1, "Only one audit should not be added");

    let fetched_audit_name = fetched_audit_data[0].get::<String, _>("name");
    let fetched_audit_name_web = fetched_audit_data_web[0].get::<String, _>("name");

    assert!(fetched_audit_name != fetched_audit_name_web, "Two audits should be different");

    assert!(audit.name == fetched_audit_name, "Audit name does not equal queried Audit name");

    assert!(
        web_audit.name == fetched_audit_name_web,
        "Web Audit name does not equal queried Web Audit name"
    );

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

    let fetched_contract_ids = fetched_contract_data
        .iter()
        .map(|contract| contract.0.clone())
        .collect::<Vec<String>>();

    let is_contract_added = audit.contracts
        .iter()
        .all(|contract| fetched_contracts.contains(contract));

    assert!(is_contract_added, "Missing contracts");

    let is_contract_added_web = web_audit.contracts
        .iter()
        .all(|contract| fetched_contracts.contains(contract));

    assert!(is_contract_added_web, "Missing contracts web audit");

    let fetched_audit_id = fetched_audit_data[0].get::<String, _>("id");

    let fetched_contracts_from_mapping = sqlx
        ::query("SELECT B FROM _AuditToContract WHERE A=?")
        .bind(fetched_audit_id)
        .fetch_all(&pool).await?;

    let is_mapping_exists = fetched_contracts_from_mapping
        .iter()
        .all(|contract_id| fetched_contract_ids.contains(&contract_id.get::<String, _>("B")));

    assert!(is_mapping_exists, "Missing mapping between audit to contract");

    let fetched_audit_id = fetched_audit_data_web[0].get::<String, _>("id");

    let fetched_contracts_from_mapping = sqlx
        ::query("SELECT B FROM _AuditToContract WHERE A=?")
        .bind(fetched_audit_id)
        .fetch_all(&pool).await?;

    let is_mapping_exists = fetched_contracts_from_mapping
        .iter()
        .all(|contract_id| fetched_contract_ids.contains(&contract_id.get::<String, _>("B")));

    assert!(is_mapping_exists, "Missing mapping between audit to contract");

    let fetched_audit_id = fetched_audit_data[0].get::<String, _>("id");

    let fetched_issue_count = sqlx
        ::query_as::<_, SQLIssueTempResult>(
            r#"
        SELECT status, severity, COUNT(*) as count
        FROM Issue
        WHERE auditId=?
        GROUP BY status, severity
        ORDER BY status, severity
        "#
        )
        .bind(fetched_audit_id)
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

    let fetched_audit_id = fetched_audit_data_web[0].get::<String, _>("id");

    let fetched_issue_count_web = sqlx
        ::query_as::<_, SQLIssueTempResult>(
            r#"
        SELECT status, severity, COUNT(*) as count
        FROM Issue
        WHERE auditId=?
        GROUP BY status, severity
        ORDER BY status, severity
        "#
        )
        .bind(fetched_audit_id)
        .fetch_all(&pool).await?;

    let (fixed, risk_accepted) = fetched_issue_count_web
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

    let fetched_issue_count_web = IssueCount::new(fixed, risk_accepted);

    assert!(web_audit.issues == fetched_issue_count_web, "Issue counts are not equal web");

    Ok(())
}