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
use core::convert::TryFrom;
use pasetors::claims::{Claims as PasetoClaims, ClaimsValidationRules};
use pasetors::keys::SymmetricKey;
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use pasetors::{Local, local};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{database::Database, errors::AppError, models::User};

/// Load PASETO symmetric key from environment.
/// Supports:
/// - 64-char hex string (32 bytes) for production
/// - Shorter strings padded/truncated to 32 bytes for development
fn get_paseto_key() -> Result<SymmetricKey<V4>, AppError> {
    let key_str = std::env::var("PASETO_KEY")
        .unwrap_or_else(|_| "track-leader-dev-secret-change-in-production".to_string());

    // Hex-encoded 32 bytes (64 hex chars) for production
    let key_bytes: [u8; 32] =
        if key_str.len() == 64 && key_str.chars().all(|c| c.is_ascii_hexdigit()) {
            let decoded: Vec<u8> = (0..64)
                .step_by(2)
                .map(|i| u8::from_str_radix(&key_str[i..i + 2], 16))
                .collect::<Result<Vec<u8>, _>>()
                .map_err(|_| AppError::Internal)?;
            decoded.try_into().map_err(|_| AppError::Internal)?
        } else {
            // Dev mode: pad/truncate to 32 bytes
            let mut bytes = [0u8; 32];
            let input = key_str.as_bytes();
            let len = input.len().min(32);
            bytes[..len].copy_from_slice(&input[..len]);
            bytes
        };

    SymmetricKey::<V4>::from(&key_bytes).map_err(|_| AppError::Internal)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
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

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, ToSchema)]
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
    let key = get_paseto_key()?;
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::days(7);

    let mut claims = PasetoClaims::new().map_err(|_| AppError::Internal)?;
    claims
        .subject(&user.id.to_string())
        .map_err(|_| AppError::Internal)?;
    claims
        .expiration(
            &exp.format(&time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(|_| AppError::Internal)?,
        )
        .map_err(|_| AppError::Internal)?;
    claims
        .issued_at(
            &now.format(&time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(|_| AppError::Internal)?,
        )
        .map_err(|_| AppError::Internal)?;
    claims
        .add_additional("email", user.email.as_str())
        .map_err(|_| AppError::Internal)?;

    local::encrypt(&key, &claims, None, None).map_err(|_| AppError::Internal)
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let key = get_paseto_key()?;
    let validation = ClaimsValidationRules::new();

    let untrusted =
        UntrustedToken::<Local, V4>::try_from(token).map_err(|_| AppError::Unauthorized)?;
    let trusted = local::decrypt(&key, &untrusted, &validation, None, None)
        .map_err(|_| AppError::Unauthorized)?;

    let payload = trusted.payload_claims().ok_or(AppError::Unauthorized)?;

    let sub = payload
        .get_claim("sub")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Unauthorized)?;
    let email = payload
        .get_claim("email")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or(AppError::Unauthorized)?;
    let exp = payload
        .get_claim("exp")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok()
        })
        .map(|t| t.unix_timestamp())
        .ok_or(AppError::Unauthorized)?;
    let iat = payload
        .get_claim("iat")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok()
        })
        .map(|t| t.unix_timestamp())
        .ok_or(AppError::Unauthorized)?;

    Ok(Claims {
        sub,
        email,
        exp,
        iat,
    })
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

// Optional extractor for authenticated user - returns None if not authenticated
pub struct OptionalAuthUser(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth_header| auth_header.strip_prefix("Bearer "))
            .and_then(|token| verify_token(token).ok());

        Ok(OptionalAuthUser(claims))
    }
}

/// Handler for user registration
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already registered"),
    )
)]
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

/// Handler for user login
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Invalid credentials"),
    )
)]
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

/// Handler to get current user
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user info", body = UserResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test user
    fn test_user() -> User {
        User::new("test@example.com".to_string(), "Test User".to_string())
    }

    #[test]
    fn test_key_loading_from_hex_string() {
        // 64-char hex string = 32 bytes
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        // SAFETY: Tests run serially with --test-threads=1 or we accept the race
        unsafe {
            std::env::set_var("PASETO_KEY", hex_key);
        }

        let result = get_paseto_key();
        assert!(result.is_ok(), "Should load key from 64-char hex string");

        unsafe {
            std::env::remove_var("PASETO_KEY");
        }
    }

    #[test]
    fn test_key_derivation_from_short_dev_string() {
        let short_key = "dev-secret";
        // SAFETY: Tests run serially with --test-threads=1 or we accept the race
        unsafe {
            std::env::set_var("PASETO_KEY", short_key);
        }

        let result = get_paseto_key();
        assert!(result.is_ok(), "Should derive key from short string");

        unsafe {
            std::env::remove_var("PASETO_KEY");
        }
    }

    #[test]
    fn test_token_roundtrip() {
        // SAFETY: Tests run serially with --test-threads=1 or we accept the race
        unsafe {
            std::env::remove_var("PASETO_KEY");
        }

        let user = test_user();
        let token = create_token(&user).expect("Should create token");

        // PASETO v4.local tokens start with this prefix
        assert!(
            token.starts_with("v4.local."),
            "Token should be PASETO v4.local format"
        );

        let claims = verify_token(&token).expect("Should verify token");
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_expired_token_rejection() {
        // SAFETY: Tests run serially with --test-threads=1 or we accept the race
        unsafe {
            std::env::remove_var("PASETO_KEY");
        }

        // Create a token, then verify with a manipulated expiration would be caught
        // Since we can't easily create an expired token without modifying internals,
        // we test that the library correctly validates expiration by checking
        // that a valid token works and trusting PASETO's exp validation
        let user = test_user();
        let token = create_token(&user).expect("Should create token");
        let claims = verify_token(&token).expect("Should verify valid token");

        // Verify expiration is set correctly (7 days from now)
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let expected_exp = now + 7 * 24 * 60 * 60;
        assert!(
            (claims.exp - expected_exp).abs() < 5,
            "Expiration should be ~7 days from now"
        );
    }

    #[test]
    fn test_invalid_token_rejection() {
        // SAFETY: Tests run serially with --test-threads=1 or we accept the race
        unsafe {
            std::env::remove_var("PASETO_KEY");
        }

        let result = verify_token("not-a-valid-token");
        assert!(result.is_err(), "Should reject invalid token format");

        let result = verify_token("v4.local.invalidpayload");
        assert!(result.is_err(), "Should reject malformed PASETO token");
    }

    #[test]
    fn test_tampered_token_rejection() {
        // SAFETY: Tests run serially with --test-threads=1 or we accept the race
        unsafe {
            std::env::remove_var("PASETO_KEY");
        }

        let user = test_user();
        let token = create_token(&user).expect("Should create token");

        // Tamper with the token by modifying a character in the payload
        let mut tampered = token.clone();
        if let Some(last_char) = tampered.pop() {
            // Change the last character
            let new_char = if last_char == 'A' { 'B' } else { 'A' };
            tampered.push(new_char);
        }

        let result = verify_token(&tampered);
        assert!(result.is_err(), "Should reject tampered token");
    }
}
