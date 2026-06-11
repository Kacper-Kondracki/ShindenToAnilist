use reqwest::Client;
use shinden_to_anilist_grpc::{
    pb::shinden_to_anilist_service_server::ShindenToAnilistServiceServer,
    server::ShindenToAnilist,
};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{
    EnvFilter,
    fmt,
};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("shinden_to_anilist_grpc=info,tower_http=info"));

    fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let addr = "127.0.0.1:50051".parse()?;
    let service = ShindenToAnilist::new(Client::new());

    info!(%addr, "ShindenToAnilist gRPC listening");

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::permissive())
        .layer(GrpcWebLayer::new())
        .add_service(ShindenToAnilistServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
