/// OPC CLI main entry point
/// Usage: opc <COMMAND> [OPTIONS]
use clap::Parser;

#[tokio::main]
async fn main() {
    opc::install_crypto_provider();
    let cli = opc::cli::Cli::parse();
    if let Err(e) = opc::cli::run(cli).await {
        eprintln!("❌ {}", e);
        std::process::exit(1);
    }
}
