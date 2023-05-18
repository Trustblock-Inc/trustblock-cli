pub mod constants;
pub mod utils;
pub mod mock_data;

use std::path::PathBuf;

use mock_data::MockAudit;
use utils::{ generate_random_audit, generate_random_pdf };

pub fn generate_random_data(
    project_seed: Option<u64>
) -> eyre::Result<(PathBuf, PathBuf, MockAudit)> {
    let pdf_file = generate_random_pdf()?;

    let (audit_file, audit) = generate_random_audit(project_seed)?;

    Ok((pdf_file, audit_file, audit))
}