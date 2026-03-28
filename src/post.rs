use crate::*;

use log::{error, info};
use std::error::Error;
use actix_web::HttpRequest;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Claims {
    sub: String,  // email or user ID
    exp: usize, //experiation
}

#[derive(Serialize)]
struct ProfileInfo {
    name: String,
    email: String,
}


#[get("/self")]
pub async fn get_personal_info(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    //extract JWT from cookie
    let jwt_cookie = req.cookie("token").ok_or_else(|| {
        error!("Missing JWT cookie from request.");
        actix_web::error::ErrorUnauthorized("Missing auth token cookie")
    })?;

    let jwt = jwt_cookie.value().to_string();

    // decode and verify JWT
    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(jwt::secret),
        &Validation::new(Algorithm::HS256),
    ).map_err(|err| {
        error!("JWT decode error: {:?}", err);
        actix_web::error::ErrorUnauthorized("Invalid JWT")
    })?;

    let claims = decoded.claims;

    // parse UUID from claims
    let public_id = Uuid::parse_str(&claims.sub).map_err(|err| {
        error!("Invalid UUID in claims.sub '{}': {:?}", claims.sub, err);
        actix_web::error::ErrorUnauthorized("Malformed UUID in token")
    })?;

    // query the DB
    let row = sqlx::query!(
        "SELECT name, email FROM \"user\" WHERE public_id = $1",
        public_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|err| {
        error!("DB query failed for user {}: {:?}", public_id, err);
        actix_web::error::ErrorInternalServerError("User not found or DB error")
    })?;

    // assemble profile info
    let user_data = ProfileInfo {
        name: row.name.unwrap_or_else(|| {
            error!("Missing name for user {}", public_id);
            "Unnamed User".to_string()
        }),
        email: row.email.unwrap_or_else(|| {
            error!("Missing email for user {}", public_id);
            "unknown@example.com".to_string()
        }),
    };

    info!("Fetched profile for user {}", public_id);
    Ok(HttpResponse::Ok().json(user_data))
}




#[post("/api/signup")]
pub async fn signup(info: web::Json<LoginData>, pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let email = info.email.trim();
    let password = info.password.trim(); 
    let username = info.username.trim();
    let public_id = Uuid::new_v4();
    let token = jwt::generate_JWT(public_id.to_string());

    let cookie = Cookie::build("token", token)
        .path("/")
        .max_age(time::Duration::days(1))
        .http_only(true)
        .same_site(SameSite::None)
        //.secure(true)
        .finish();

    let db_url = env::var("DATABASE_URL")
        .map_err(|_| actix_web::error::ErrorInternalServerError("Missing DB URL"))?;

    let email_exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM \"user\" WHERE email = $1)")
        .bind(email)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| {
            let err_msg = format!("{}", e);   
            eprintln!("DB query error: {:?}", err_msg);
            actix_web::error::InternalError::from_response(
                e, 
                HttpResponse::InternalServerError().json(LoginResponse {
                    success: false,
                    message: err_msg, 
                }),
            )
        })?;

    if email_exists {
        return Ok(HttpResponse::BadRequest().json(LoginResponse {
            success: false,
            message: "ExistingEmail".to_string(),
        }));
    }
   
    //auto generates id
    sqlx::query("INSERT INTO \"user\" (name, email, password, public_id) VALUES ($1, $2, $3, $4)")
        .bind(username)
        .bind(email)
        .bind(password)
        .bind(public_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            let msg =  format!("{}", e);
            println!("{}", msg);
            actix_web::error::InternalError::from_response(
            e,
            HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                message: "Insert failed".to_string(),
            }),
        )})?;

    Ok(
        HttpResponse::Ok()
        .cookie(cookie)
        .json(LoginResponse {
            success: true,
            message: "Signup successful".to_string(),
        })
    )
}


