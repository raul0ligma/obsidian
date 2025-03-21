use actix_web::web;

use crate::order_processor;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1").route("/order", web::post().to(order_processor::process_order)));
}
