use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{self, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Panic if we can't get the configuarion
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::log::info!("Listening on {}", address);
    let listener = TcpListener::bind(address).expect("Failed to port a random port");
    // Connection
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    run(listener, connection_pool)?.await
}
