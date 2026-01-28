use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

use crate::{database::Database, errors::AppError, models::User};

// JWT secret - in production, load from environment
fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "track-leader-dev-secret-change-in-production".to_string())
        .into_bytes()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
        }
    }
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InvalidInput(format!("Failed to hash password: {e}")))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::InvalidInput(format!("Invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_token(user: &User) -> Result<String, AppError> {
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::days(7);

    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        exp: exp.unix_timestamp(),
        iat: now.unix_timestamp(),
    };

    let secret = get_jwt_secret();
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|_| AppError::Internal)
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let secret = get_jwt_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(token_data.claims)
}

// Extractor for authenticated user
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_token(token)?;
        Ok(AuthUser(claims))
    }
}

// Handler for user registration
pub async fn register(
    Extension(db): Extension<Database>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate input using validator crate
    req.validate().map_err(|e| {
        let messages: Vec<String> = e
            .field_errors()
            .into_iter()
            .flat_map(|(_, errors)| {
                errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
            })
            .collect();
        AppError::InvalidInput(messages.join(", "))
    })?;

    // Check if user exists
    if db.get_user_by_email(&req.email).await?.is_some() {
        return Err(AppError::InvalidInput(
            "Email already registered".to_string(),
        ));
    }

    // Hash password and create user
    let password_hash = hash_password(&req.password)?;
    let user = User::new(req.email, req.name);

    db.create_user_with_password(&user, &password_hash).await?;

    let token = create_token(&user)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

// Handler for user login
pub async fn login(
    Extension(db): Extension<Database>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate input
    req.validate().map_err(|e| {
        let messages: Vec<String> = e
            .field_errors()
            .into_iter()
            .flat_map(|(_, errors)| {
                errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
            })
            .collect();
        AppError::InvalidInput(messages.join(", "))
    })?;

    let (user, password_hash) = db
        .get_user_with_password(&req.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let Some(hash) = password_hash else {
        return Err(AppError::Unauthorized);
    };

    if !verify_password(&req.password, &hash)? {
        return Err(AppError::Unauthorized);
    }

    let token = create_token(&user)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

// Handler to get current user
pub async fn me(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = db
        .get_user_by_email(&claims.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(user.into()))
}

#[derive(Debug)]
pub struct Unauthorized;

impl IntoResponse for Unauthorized {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}
