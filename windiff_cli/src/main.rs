mod cli;
mod configuration;
mod database;
mod download;
mod error;
mod pdb;
mod resym_frontend;
mod winbindex;

use env_logger::Env;
use structopt::StructOpt;

use crate::{
    cli::WinDiffOpt,
    configuration::WinDiffConfiguration,
    database::generate_databases,
    download::{download_binaries, download_pdbs},
    error::Result,
};

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(
        Env::default().default_filter_or(format!("{}=info", PACKAGE_NAME)),
    )
    .init();

    // Parse command-line options
    let opt = WinDiffOpt::from_args();
    log::info!("Using configuration file: {:?}", opt.configuration);

    // Parse configuration file
    let cfg = WinDiffConfiguration::from_file(&opt.configuration).await?;

    // Download requested PEs
    let tmp_directory = tempdir::TempDir::new(PACKAGE_NAME)?;
    let output_directory = tmp_directory.path();
    log::info!("Downloading PEs...");
    let downloaded_pes = download_binaries(&cfg, output_directory).await?;
    log::trace!("PEs downloaded!");

    // Download PDBs
    log::info!("Downloading PDBs...");
    let downloaded_binaries = download_pdbs(downloaded_pes, output_directory).await;
    log::trace!("PDBs downloaded!");

    // Extract information from PEs
    log::info!("Generating databases...");
    generate_databases(&cfg, &downloaded_binaries, &opt.output_directory).await?;
    log::info!(
        "Databases have been generated at {:?}",
        opt.output_directory
    );

    Ok(())
}
