use ethers::{
    middleware::SignerMiddleware,
    prelude::k256::ecdsa::SigningKey,
    providers::{ Http, Middleware, Provider },
    signers::{ LocalWallet, Signer, Wallet },
    types::U256,
};

use std::{ fs::File, io::BufReader, path::PathBuf, sync::Arc };

use validator::{ validate_email, validate_url };

use eyre::{ eyre, ContextCompat };

use serde::de::DeserializeOwned;

use pdf::file::FileOptions as PdfFile;

use crate::constants::CLI_PATH;

pub type WalletSignerMiddleware = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

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

pub async fn get_client(
    rpc_url: &str,
    private_key: Option<String>
) -> eyre::Result<(Arc<WalletSignerMiddleware>, U256, Wallet<SigningKey>)> {
    let home_dir = dirs::home_dir().wrap_err("Could not find home directory")?;

    let env_path = home_dir.join(format!("{CLI_PATH}/.env"));

    dotenv::from_path(env_path.as_path())?;

    let wallet_key = match private_key {
        Some(key) => key,
        None => std::env::var("WALLET_KEY")?,
    };

    let provider = Provider::<Http>::try_from(rpc_url)?;

    let chain_id = provider.get_chainid().await?;

    // this wallet's private key
    let wallet = wallet_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());

    let provider = SignerMiddleware::new(provider, wallet.clone());

    let client = Arc::new(provider);
    Ok((client, chain_id, wallet))
}