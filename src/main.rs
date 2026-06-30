mod database;
mod error;
mod domain;
mod infrastructure;
mod application;

use axum::{Json, Router, extract::{Query, State}, http::StatusCode, routing::{get, post}};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use tokio::net::TcpListener;



use database::postgres::system::db;


#[tokio::main]
async fn main() {
    
    let pool = db::connect_db().await;

    // define router
    let router01 = Router::new()
        .route("/", get(root_fn))
        .route("/test_fetch_api", get(fetch_data_api))
        .route("/test_create_api", post(create_data_api))
        .with_state(pool.clone());

    // define address
    let address: &'static str = "0.0.0.0:8000";

    // define listener
    let listener = TcpListener::bind(&address).await.unwrap();

    println!("Server running on port 8000!");

    // run router and listener
    axum::serve(listener, router01)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();


    println!("Database connections closing...");
    pool.close().await; 
    println!("Application fully shut down. Goodbye!");

}

#[derive(Serialize, Debug, FromRow)]
pub struct Data {
    id: uuid::Uuid,
    car_name: String,
    car_model: String,
    car_manufacturer: String,
    production_year: i32,
}

#[derive(Deserialize, Debug)]
pub struct CreateCarPayload {
    car_name: String,
    car_model: String,
    car_manufacturer: String,
    production_year: i32
}


#[derive(Serialize, Debug)]
pub struct Response {
    message: String
}


#[derive(Deserialize)]
struct Params {
    production_year: i32
}

async fn shutdown_signal() {
    _ = tokio::signal::ctrl_c()
        .await
        .expect("\nCtrl+C received! Shutting down async tasks.")
}


async fn root_fn() -> &'static str {
    "Welcome to car management API!"
}



async fn fetch_data_api(
    State(pool): State<PgPool>,
    Query(params): Query<Params>
) -> Result<Json<Vec<Data>>, StatusCode>{
    let cars = sqlx::query_as::<_, Data>(
        "
            SELECT id, car_name, car_model, car_manufacturer, production_year
            FROM item.car 
            WHERE production_year = $1
        "
    )
    .bind(params.production_year)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("There is no data with id: {} {:?}", params.production_year, e);
        StatusCode::NOT_FOUND
    })?;

    Ok(Json(cars))
}



// use transaction so when fail all fail to be inserted
async fn create_data_api(
    State(pool): State<PgPool>, 
    Json(payload): Json<Vec<CreateCarPayload>>
) -> Result<(StatusCode, Json<Vec<Data>>), (StatusCode, Json<Response>)> {

    let mut tx = pool.begin().await.map_err(|e| {
        eprintln!("Tx error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Response {message: "DB error".to_string()}))
    })?;

    let mut inserted  = Vec::with_capacity(payload.len());

    for car in payload {
        let row = sqlx::query_as::<_, Data>(
            "INSERT INTO item.car (car_name, car_model, car_manufacturer, production_year)
                VALUES ($1, $2, $3, $4)
                RETURNING id, car_name, car_model, car_manufacturer, production_year
            "
        )
        .bind(car.car_name)
        .bind(car.car_model)
        .bind(car.car_manufacturer)
        .bind(car.production_year)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Insert error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Response{message: "Failed to create car".to_string()}))
        })?;

        inserted.push(row);
    }
    
    tx.commit().await.map_err(|e| {
        eprintln!("Commit error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Response{message: "Failed to commit".to_string()}))
    })?;

    Ok((StatusCode::CREATED, Json(inserted)))

}