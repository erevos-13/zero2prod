use actix_web::{web, App, HttpRequest, HttpResponse, Responder};
use env_logger::Env;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::io::Error;
use std::net::TcpListener;
use tracing::dispatcher::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry};
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("Hello {}", &name)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let sub = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(sub);

    let config = get_configuration().expect("Fail to read configuration.");
    let connection_string = config.database.connection_string();
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind port");
    let conn_pool = PgPool::connect(&connection_string.expose_secret())
        .await
        .expect("fail to connect to DB");
    run(listener, conn_pool.clone())?.await
}
