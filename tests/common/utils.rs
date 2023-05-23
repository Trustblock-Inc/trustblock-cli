use std::{ fs::File, iter::repeat_with, path::PathBuf };

use crate::common::{ constants::{ FONT_DIR, PDF_REPORTS_PATH }, mock_data::MockAudit };
use ethers::abi::Address;
use sqlx::{ mysql::MySqlPoolOptions, MySql, Pool };
use strum::IntoEnumIterator;

use trustblock_cli::types::{ AuditContract, Chains, IssueCount, Project, SeverityCount };

use tempfile::NamedTempFile;

use super::constants::AUDIT_JSON_PATH;

pub fn generate_random_pdf() -> eyre::Result<PathBuf> {
    let default_font_name = "LiberationSans";

    let pdf_file = NamedTempFile::new_in(PDF_REPORTS_PATH)?;

    let random_string: String = repeat_with(fastrand::alphanumeric).take(10).collect();

    let default_font = genpdf::fonts::from_files(
        FONT_DIR,
        default_font_name,
        Some(genpdf::fonts::Builtin::Helvetica)
    )?;

    let mut doc = genpdf::Document::new(default_font);
    // Change the default settings
    doc.set_title("Audit Report");
    // Customize the pages
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);
    // Add one or more elements
    doc.push(
        genpdf::elements::Paragraph::new(
            format!("This is an audit report. Description: {}", random_string)
        )
    );

    // Render the document and write it to a file
    doc.render_to_file(&pdf_file)?;

    let pdf_file = pdf_file.into_temp_path().keep()?;

    Ok(pdf_file)
}

pub fn generate_random_audit(project_seed: Option<u64>) -> eyre::Result<(PathBuf, MockAudit)> {
    let file = File::open(AUDIT_JSON_PATH)?;
    let default_audit = serde_json::from_reader::<File, MockAudit>(file)?;

    let chains = Chains::iter().collect::<Vec<Chains>>();

    let random_name: String = repeat_with(fastrand::alphanumeric).take(10).collect();

    let severity_count_risk_accepted = SeverityCount::new(
        fastrand::u8(..5),
        fastrand::u8(..5),
        fastrand::u8(..5),
        fastrand::u8(..5)
    );

    let severity_count_fixed = SeverityCount::new(
        fastrand::u8(..5),
        fastrand::u8(..5),
        fastrand::u8(..5),
        fastrand::u8(..5)
    );

    let issues = IssueCount::new(severity_count_fixed, severity_count_risk_accepted);

    let contracts = repeat_with(|| {
        AuditContract::new(chains[fastrand::usize(..chains.len())], Address::random())
    })
        .take(5)
        .collect::<Vec<AuditContract>>();

    let project = generate_random_project(project_seed)?;

    let audit_struct = MockAudit {
        project,
        issues,
        contracts,
        name: random_name,

        ..default_audit
    };

    let audit_file = NamedTempFile::new()?;

    serde_json::to_writer(&audit_file, &audit_struct)?;

    let audit_file = audit_file.into_temp_path().keep()?;

    Ok((audit_file, audit_struct))
}

pub fn generate_random_project(seed: Option<u64>) -> eyre::Result<Project> {
    if let Some(seed) = seed {
        fastrand::seed(seed);
    }

    let name: String = repeat_with(fastrand::alphanumeric).take(10).collect();

    let twitter = format!("https://twitter.com/{name}");

    let github = format!("https://github.com/{name}");

    let website = format!("https://{name}.com");

    let email = format!("{name}@mail.com");

    let project = Project::new(name, Some(twitter), Some(github), Some(website), Some(email), None);

    Ok(project)
}

pub async fn connect_db() -> eyre::Result<Pool<MySql>> {
    let url = "mysql://user:pass@localhost:3306/local";

    let pool = MySqlPoolOptions::new().max_connections(5).connect(url).await?;

    Ok(pool)
}

pub fn clean_temp_file(path: PathBuf) -> eyre::Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}