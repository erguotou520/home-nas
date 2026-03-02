use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web::error::ErrorUnauthorized;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,  // user id
    pub username: String,
    pub role: String,
    pub exp: usize,
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

impl actix_web::FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // Get token from Authorization header
        let auth_header = req.headers().get("Authorization");
        
        let token = match auth_header {
            Some(value) => {
                let value_str = value.to_str().unwrap_or("");
                if value_str.starts_with("Bearer ") {
                    value_str.trim_start_matches("Bearer ").to_string()
                } else {
                    return ready(Err(ErrorUnauthorized("Invalid authorization header")));
                }
            }
            None => return ready(Err(ErrorUnauthorized("Missing authorization header"))),
        };
        
        // Get JWT secret from app data
        let jwt_secret = match req.app_data::<actix_web::web::Data<String>>() {
            Some(secret) => secret.as_str().to_string(),
            None => return ready(Err(ErrorUnauthorized("Server configuration error"))),
        };
        
        // Decode and validate token
        match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(token_data) => ready(Ok(token_data.claims)),
            Err(_) => ready(Err(ErrorUnauthorized("Invalid token"))),
        }
    }
}

pub fn create_token(user_id: i32, username: &str, role: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp: expiration,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
