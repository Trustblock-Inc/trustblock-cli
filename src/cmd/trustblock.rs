use crate::cmd::{ clean::CleanArgs, init::InitArgs, publish_audit::PublishAuditArgs };

use clap::{ command, Parser, Subcommand };

use std::str;

#[derive(Debug, Parser)]
#[command(about = "Trustblock CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(
        about = "Publishes an audit to Trustblock",
        arg_required_else_help = true,
        next_line_help = true
    )] PublishAudit(PublishAuditArgs),

    #[command(about = "Initializes .trustblock folder")] Init(InitArgs),

    #[command(about = "Cleans .trustblock folder")] Clean(CleanArgs),
}