use actix_web::{get, post, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use openssl::ssl::{SslAcceptor, SslMethod, SslFiletype};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use actix_web::web;
use jsonwebtoken::TokenData;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::error::ErrorInternalServerError;
use sqlx::PgPool;
use actix_web::middleware::Logger;

mod jwt;
mod post;
mod user;

#[derive(Serialize)]
struct ConnectionResponse {
    success: bool,
    message: String,
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenvy::dotenv().expect("Failed to read .env file");
    println!("check: {:?}", jwt::verify_jwt(&jwt::generate_JWT("some_email".to_string())));
    
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let ip = "127.0.0.1";
    let port = 8080;
   
    println!("http://{}:{}", ip, port);
        

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .map_err(|e| actix_web::error::InternalError::from_response(
            e,
            HttpResponse::InternalServerError().json(ConnectionResponse {
                success: false,
                message: "DB connection failed".to_string(),
            }),
        )).unwrap();



    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .unwrap();
    
    println!("active on: {ip}{port}");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .supports_credentials() 
                    .allowed_origin("http://localhost:5173")
            )
            .app_data(web::Data::new(pool.clone()))  // Add pool here once
            .service(hello) //get
            .service(jwt::JWT_test)
            .service(post::get_user_uuid)
    })
    .bind((ip, port))?
    .run()
    .await
}

