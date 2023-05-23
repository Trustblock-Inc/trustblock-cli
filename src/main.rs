use clap::Parser;

use trustblock_cli::{
    cmd::{ block_on, trustblock::{ Cli, Commands }, Cmd, check_update },
    constants::CLI_PATH,
    error_handler,
};

fn main() -> eyre::Result<()> {
    error_handler::install()?;
    let cli = Cli::parse();

    block_on(check_update())?;

    match cli.command {
        Commands::PublishAudit(cmd) => {
            println!("Publishing an audit\n");
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

#[cfg(test)]
mod tests {
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        use super::*;

        Cli::command().debug_assert();
    }
}