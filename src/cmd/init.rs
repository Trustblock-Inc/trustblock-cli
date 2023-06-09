use crate::{ cmd::utils::Cmd, constants::CLI_PATH };

use std::{ fs::File, io::prelude::Write };

use eyre::ContextCompat;

use clap::Parser;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Parser)]
pub struct InitArgs {
    #[clap(
        short,
        long,
        help = "Trustblock API key to add to .env",
        long_help = "Trustblock API key, which you can get in your profile. If supplied, it will be added automatically to .env"
    )]
    api_key: Option<String>,
}

impl Cmd for InitArgs {
    fn run(self) -> eyre::Result<()> {
        let home_dir = dirs::home_dir().wrap_err("Could not find home directory")?;

        let api_key = self.api_key.unwrap_or_default();

        // Create the path to the .trustblock directory
        let cli_dir = home_dir.join(CLI_PATH);

        // Create the .trustblock directory if it doesn't exist
        if !cli_dir.exists() {
            std::fs::create_dir(&cli_dir)?;
        }

        // Create the path to the .env file
        let env_path = cli_dir.join(".env");

        if env_path.exists() {
            println!(".env file already exists at {env_path:?}");
            return Ok(());
        }

        let mut env_file = File::create(&env_path)?;

        let env_data = format!("API_KEY={api_key}");

        env_file.write_all(env_data.as_bytes())?;

        println!("Created .env file at {env_path:?}");

        Ok(())
    }
}