// use sqlx::{Connection, PgConnection};
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]

async fn main() -> std::io::Result<()> {
    // let listener =
    //     TcpListener::bind("127.0.0.1:0").expect("Faileea at Main to create listener & bind port");
    // run(listener)?.await
    let configuration = get_configuration().expect("Failed to read configuration");
    // let connection = PgConnection::connect(&configuration.database.connection_string())
    // .await
    // .expect("Failed to conneect in Postgres in my main.rs");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres using PgPool");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
