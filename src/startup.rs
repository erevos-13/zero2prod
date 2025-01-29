use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use sqlx::{PgConnection, PgPool};
use std::{io::Error, net::TcpListener};

pub fn run(listener: TcpListener, conn: PgPool) -> Result<Server, Error> {
    let connection = web::Data::new(conn);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
