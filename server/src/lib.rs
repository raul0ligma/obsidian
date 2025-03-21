pub mod order_processor;
pub mod router;
pub mod server;

struct Config {
    host: String,
    port: u16,
}
