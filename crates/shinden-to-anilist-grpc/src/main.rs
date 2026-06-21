use std::{
    env,
    io::{
        self,
        Read,
    },
    net::SocketAddr,
    thread,
};

use shinden_to_anilist_grpc::{
    pb::shinden_to_anilist_service_server::ShindenToAnilistServiceServer,
    server::ShindenToAnilist,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{
    EnvFilter,
    fmt,
};

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:45187";
const LISTEN_ADDR_ENV: &str = "SHINDEN_TO_ANILIST_GRPC_LISTEN_ADDR";
const EXIT_ON_STDIN_CLOSE_ENV: &str = "SHINDEN_TO_ANILIST_GRPC_EXIT_ON_STDIN_CLOSE";

#[derive(Debug)]
struct Config {
    listen_addr: SocketAddr,
    exit_on_stdin_close: bool,
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("shinden_to_anilist_core=info,shinden_to_anilist_grpc=info,tower_http=info")
    });

    fmt().with_env_filter(filter).init();
}

fn config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut listen_addr = env::var(LISTEN_ADDR_ENV).unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_owned());
    let mut exit_on_stdin_close = env::var(EXIT_ON_STDIN_CLOSE_ENV)
        .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"));
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--listen-addr" {
            listen_addr = args
                .next()
                .ok_or_else(|| "--listen-addr requires an address".to_owned())?;
            continue;
        }

        if let Some(addr) = arg.strip_prefix("--listen-addr=") {
            listen_addr = addr.to_owned();
            continue;
        }

        if arg == "--exit-on-stdin-close" {
            exit_on_stdin_close = true;
        }
    }

    Ok(Config {
        listen_addr: listen_addr.parse()?,
        exit_on_stdin_close,
    })
}

fn stdin_close_shutdown() -> tokio::sync::watch::Receiver<bool> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    thread::spawn(move || {
        let mut stdin = io::stdin().lock();
        let mut buffer = [0u8; 1];

        loop {
            match stdin.read(&mut buffer) {
                Ok(0) | Err(_) => {
                    let _ = shutdown_tx.send(true);
                    break;
                },
                Ok(_) => {},
            }
        }
    });

    shutdown_rx
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = config()?;
    let addr = config.listen_addr;
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    let service = ShindenToAnilist::new()?;
    let stdin_shutdown = config.exit_on_stdin_close.then(stdin_close_shutdown);

    info!(%addr, "ShindenToAnilist gRPC listening");
    println!(r#"{{"event":"ready","addr":"{addr}"}}"#);

    let server = Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::permissive())
        .layer(GrpcWebLayer::new())
        .add_service(ShindenToAnilistServiceServer::new(service));

    if let Some(mut shutdown) = stdin_shutdown {
        server
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                let _ = shutdown.changed().await;
                info!("ShindenToAnilist gRPC shutting down after stdin closed");
            })
            .await?;
    } else {
        server
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await?;
    }

    Ok(())
}
