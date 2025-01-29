use actix_web::{web, HttpResponse};
use chrono::Utc;
use reqwest::Request;
use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    // let request_id = Uuid::new_v4();
    // let request_span = tracing::trace_span!("Adding a new subscriber", %request_id, subscriber_email = %form.email,subscriber_name = %form.name );
    // tracing::info!(
    //     "request id: {}, Adding '{}' '{}' as a new subscriber",
    //     request_id,
    //     form.email,
    //     form.name
    // );
    // let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new subscriber detain in the database");

    // tracing::info!(
    //     "request id {} - Saving new subscriber details in the database",
    //     request_id
    // );
    /// We using `r#` for leters in string so we can not escaping
    // match sqlx::query!(
    //     r#"
    //     INSERT INTO subscriptions (id,email,name,subscribed_at)
    //     VALUES ($1,$2,$3,$4)
    // "#,
    //     Uuid::new_v4(),
    //     form.email,
    //     form.name,
    //     Utc::now()
    // )
    // .execute(pool.get_ref())
    // .instrument(query_span)
    // .await
    // {
    //     Ok(_) => {
    //         // tracing::info!(
    //         //     "request_id {} -  New subscriber details have been saved",
    //         //     request_id
    //         // );
    //         HttpResponse::Ok().finish()
    //     }
    //     Err(e) => {
    //         // tracing::error!(
    //         //     "request_id  {} - Fail to execute query: {:?}",
    //         //     request_id,
    //         //     e
    //         // );
    //         HttpResponse::InternalServerError().finish()
    //     }
    // }
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new Subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id,email,name,subscribed_at)
        VALUES ($1,$2,$3,$4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Fail to execute: {:?}", e);
        e
    })?;

    Ok(())
}
