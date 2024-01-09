mod log_middleware;

use std::net::SocketAddrV4;

use clap::Parser;

use impostor_compiler_axum::compile;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PORT: u16 = 3939;

#[derive(Parser, Debug)]
#[clap(name = "Impostor", version = "0.1.0", author = "Impostor Contributors")]
struct Cli {
    /// Path to an Impostor mock file
    path_to_file: String,

    /// Port to listen on, defaults to 3939
    #[clap(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    log::info!("Impostor v{}", VERSION);

    let args = Cli::parse();
    let contents = std::fs::read_to_string(&args.path_to_file)?;

    let compiled =
        compile(&contents)?.layer(axum::middleware::from_fn(log_middleware::log_middleware));

    log::info!("Loaded file {}", &args.path_to_file);

    let port = args.port.unwrap_or(DEFAULT_PORT);
    let address = SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), port);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, compiled).await?;

    Ok(())
}
