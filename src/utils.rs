use std::{ fs::File, io::BufReader, path::PathBuf };

use validator::{ validate_email, validate_url };

use eyre::{ eyre, ContextCompat };

use serde::de::DeserializeOwned;

use pdf::file::FileOptions as PdfFile;

use crate::constants::CLI_PATH;

pub trait Pdf {
    fn pdf_file_check(&self) -> eyre::Result<&PathBuf>;
}

impl Pdf for PathBuf {
    fn pdf_file_check(&self) -> eyre::Result<&Self> {
        PdfFile::uncached().open(self)?;

        Ok(self)
    }
}

pub fn parse_json<T: DeserializeOwned>(path: &PathBuf) -> eyre::Result<T> {
    let file = File::open(path)?;

    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;

    Ok(data)
}

pub fn validate_links(link: &str) -> eyre::Result<String> {
    if !validate_url(link) {
        return Err(eyre!("Invalid URL"));
    }

    Ok(link.to_string())
}

pub fn validate_emails(email: &str) -> eyre::Result<String> {
    if !validate_email(email) {
        return Err(eyre!("Invalid Contact Email"));
    }

    Ok(email.to_string())
}

pub fn max_length_string(val: &str) -> eyre::Result<String> {
    if val.len() > 28 {
        return Err(eyre!("Name should be less than 28 characters"));
    }

    Ok(val.to_string())
}

pub fn validate_pdf(val: &str) -> eyre::Result<PathBuf> {
    let path_buf = PathBuf::from(val);

    path_buf
        .pdf_file_check()
        .map_err(|e| eyre!("Invalid PDF file. Please upload a valid PDF file: {e}"))?;

    Ok(path_buf)
}

pub fn apply_dotenv() -> eyre::Result<()> {
    let home_dir = dirs::home_dir().wrap_err("Could not find home directory")?;

    let env_path = home_dir.join(format!("{CLI_PATH}/.env"));

    Ok(dotenv::from_path(env_path.as_path())?)
}