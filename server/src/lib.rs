pub mod order_processor;
pub mod prover;
pub mod router;
pub mod server;

pub struct Config {
    host: String,
    port: u16,
    rpc_url: String,
}
