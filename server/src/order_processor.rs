use actix_web::{web, HttpResponse, Responder};
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrderRequest {
    chain_id: u64,
    address: String,
    commit_block: u64,
    swap_venue: String,
    amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrderResponse {
    proof: String,
    order_id: String,
    transaction_hash: String,
    error: Option<String>,
}

pub async fn process_order(order: web::Json<NewOrderRequest>) -> impl Responder {
    info!("new order {:?}", order);
    let response = NewOrderResponse {
        proof: "lol".into(),
        order_id: "keke".into(),
        transaction_hash: "wasd".into(),
        error: None,
    };
    debug!("sending response: {:?}", response);
    HttpResponse::Ok().json(response)
}
