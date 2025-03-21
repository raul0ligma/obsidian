use actix_cors::Cors;
use actix_web::{
    dev::Server as ActixServer, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use log::{debug, info};
use std::env;

use crate::Config;

pub type RouteConfigFn = fn(&mut web::ServiceConfig);

pub async fn health_check() -> impl Responder {
    debug!("Health check request received");
    HttpResponse::Ok().body("Server is running!")
}

pub struct Server {
    cfg: Config,
    routes: Vec<RouteConfigFn>,
}

impl Server {
    pub fn new_server_from_env() -> Self {
        dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a valid number");

        Server {
            cfg: Config {
                host,
                port,
                rpc_url,
            },
            routes: Vec::new(),
        }
    }

    pub fn add_handler(mut self, route_config: RouteConfigFn) -> Self {
        self.routes.push(route_config);
        self
    }

    pub fn run(&self) -> std::io::Result<ActixServer> {
        let address = format!("{}:{}", self.cfg.host, self.cfg.port);
        let routes = self.routes.clone();

        info!("Starting server at http://{}", address);
        let server = HttpServer::new(move || {
            let mut app = App::new()
                .wrap(Logger::new("%a \"%r\" %s %b \"%{User-Agent}i\" %D ms"))
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header()
                        .max_age(3600),
                )
                .route("/health", web::get().to(health_check));

            for route_fn in &routes {
                app = app.configure(*route_fn);
            }

            app
        })
        .bind(address)?;

        Ok(server.run())
    }
}
