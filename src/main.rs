use clap::Parser;
use dashmap::DashMap;
use tower_lsp::{LspService, Server};

use vale_ls::server::Backend;
use vale_ls::vale::ValeManager;

/// The official Vale Language Server.
#[derive(Parser, Debug)]
#[command(version)]
struct Args;

#[tokio::main]
async fn main() {
    env_logger::init();

    let _ = Args::parse();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
        param_map: DashMap::new(),
        cli: ValeManager::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
