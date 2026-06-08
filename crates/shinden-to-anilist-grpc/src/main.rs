use reqwest::Client;
use shinden_to_anilist_grpc::{
    pb::shinden_to_anilist_service_server::ShindenToAnilistServiceServer,
    server::ShindenToAnilist,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let service = ShindenToAnilist::new(Client::new());

    println!("ShindenToAnilist gRPC listening on {addr}");

    Server::builder()
        .add_service(ShindenToAnilistServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
