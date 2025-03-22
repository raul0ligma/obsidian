use actix_web::{web, HttpResponse, Responder};
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::prover::Prover;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderRequest {
    pub chain_id: u64,
    pub address: String,
    pub commit_block: u64,
    pub amount: String,
    pub pool_address: String,
    pub sell_token: String,
    pub buy_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrderResponse {
    pub block: u64,
    pub proof: String,
    pub public_values: String,
    pub error: Option<String>,
}

pub async fn process_order(order: web::Json<NewOrderRequest>) -> impl Responder {
    let prover = Prover::new();
    let proved_order = prover.prove(order.clone()).await;
    info!("new order {:?}", order.0);

    debug!("sending response: {:?}", proved_order);
    HttpResponse::Ok().json(proved_order)
}
