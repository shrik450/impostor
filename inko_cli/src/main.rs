use std::net::SocketAddrV4;

use clap::Parser;

use inko_compiler_axum::compile;

#[derive(Parser, Debug)]
#[clap(name = "Inko", version = "0.1.0", author = "Inko Contributors")]
struct Cli {
    /// Path to an Inko mock file
    path_to_file: String,

    /// Port to listen on, defaults to 3939
    #[clap(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Cli::parse();

    let contents = std::fs::read_to_string(args.path_to_file)?;

    let compiled = compile(&contents)?;

    let port = args.port.unwrap_or(3939);
    let address = SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), port);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, compiled).await?;

    Ok(())
}
