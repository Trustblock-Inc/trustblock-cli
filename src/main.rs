use clap::Parser;

use trustblock_cli::{
    cmd::{ block_on, trustblock::{ Cli, Commands }, Cmd },
    constants::CLI_PATH,
    error_handler,
};

fn main() -> eyre::Result<()> {
    error_handler::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::PublishAudit(cmd) => {
            println!("Publishing an audit");
            block_on(cmd.run())?;
            Ok(())
        }
        Commands::Init(cmd) => {
            println!("Generating {CLI_PATH} folder...\n");
            cmd.run()?;
            Ok(())
        }
        Commands::Clean(cmd) => {
            println!("Cleaning {CLI_PATH} folder...\n");
            cmd.run()?;
            Ok(())
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Cli::command().debug_assert();
}

#[cfg(test)]
mod tests {
    // use super::*;
    #[test]
    fn test() {}
}