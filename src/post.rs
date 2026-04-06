use crate::*;
use log::{error, info};
use std::error::Error;
use actix_web::HttpRequest;
use uuid::Uuid;

use crate::user::ProfileInfo;

#[derive(Debug, Deserialize)]
pub struct Claims {
    sub: String,  // email or user ID
    exp: usize, //experiation
}


#[derive(Deserialize)]
struct LoginData {
    email: String,
    password: String,
    username: String,
}

#[get("/user/{id}")]
pub async fn get_user_uuid(req: HttpRequest, pool: web::Data<PgPool>, web_path: web::Path<uuid::Uuid>) -> Result<HttpResponse, actix_web::Error> {
    // query the DB
    
    let id = web_path.into_inner();

    let row = sqlx::query!(
        "SELECT name, interests FROM \"User\" WHERE id = $1",
        id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|err| {
        error!("DB query failed for user {}: {:?}", id, err);
        actix_web::error::ErrorInternalServerError("User not found or DB error")
    })?;

    // assemble profile info
    let user_data = ProfileInfo::init(
        row.name.unwrap_or_else(|| {
            error!("Missing name for user {}", id);
            "Unnamed User".to_string()
        }),
        row.interests.unwrap_or_else(|| {
            error!("Missing interests for user {}", id);
            Vec::new()
        })
    );

    info!("Fetched profile for user {}", id);
    Ok(HttpResponse::Ok().json(user_data))
}
