use crate::*;

pub const secret:&[u8; 16] = b"super-secret-key";

pub fn verify_jwt(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
}

pub fn generate_JWT(user:String) -> String {

    // Create token claims
    let now = Utc::now();
    let claims = Claims {
        sub: user,
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(2)).timestamp() as usize,
        iss: "fitbunny".to_string(),
    };

    // Encode the token
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret)).unwrap();
    println!("Generated JWT: {}", token);

    // Decode the token
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    ).unwrap();
    token
}





#[get("/JWT")]
pub async fn JWT_test() -> impl Responder {
    // Create token claims
    let now = Utc::now();
    let claims = Claims {
        sub: "user123".to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(2)).timestamp() as usize,
        iss: "my-rust-app".to_string(),
    };

    // Encode the token
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret)).unwrap();
    println!("Generated JWT: {}", token);

    // Decode the token
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    ).unwrap();

    HttpResponse::Ok().body(format!("{:?}", token))
}

