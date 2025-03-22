use actix_web::{web, HttpResponse, Responder};
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
    log::info!(
        "received new order request for pool: {}",
        order.pool_address
    );
    log::debug!("order details: {:?}", order);

    let prover = Prover::new();

    match prover.prove(order.clone()).await {
        Ok(proved_order) => {
            log::info!(
                "successfully processed order for block {}",
                proved_order.block
            );
            log::debug!("sending response: {:?}", proved_order);
            HttpResponse::Ok().json(proved_order)
        }
        Err(error) => {
            log::error!("order processing failed: {}", error);
            let error_response = NewOrderResponse {
                block: 0,
                proof: String::new(),
                public_values: String::new(),
                error: Some(error),
            };
            HttpResponse::InternalServerError().json(error_response)
        }
    }
}
