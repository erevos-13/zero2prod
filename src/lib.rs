pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::io::Error;
use std::net::TcpListener;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
