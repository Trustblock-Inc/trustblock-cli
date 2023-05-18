use crate::{ cmd::utils::Cmd, constants::CLI_PATH };

use eyre::ContextCompat;

use clap::Parser;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Parser)]
pub struct CleanArgs {}

impl Cmd for CleanArgs {
    fn run(self) -> eyre::Result<()> {
        let home_dir = dirs::home_dir().wrap_err("Could not find home directory")?;
        let trustblock_dir = home_dir.join(CLI_PATH);

        if !trustblock_dir.exists() {
            println!("No .trustblock folder found");
            return Ok(());
        }

        std::fs::remove_dir_all(trustblock_dir)?;

        println!("Cleaned .trustblock folder");

        Ok(())
    }
}