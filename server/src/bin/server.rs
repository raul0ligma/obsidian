use log::info;
use server::{router, server::Server};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    let server = Server::new_server_from_env().add_handler(router::router);

    info!("starting server with configuration");

    server.run()?.await
}
