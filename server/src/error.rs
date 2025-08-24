// /var/www/structure/server/src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    SqlxError(sqlx::Error),
    PasswordHashError(argon2::password_hash::Error),
    JwtError(jsonwebtoken::errors::Error),
    NotFound,
    Unauthorized,
    InvalidCredentials,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::SqlxError(e) => {
                tracing::error!("SQLx error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string())
            }
            AppError::PasswordHashError(e) => {
                tracing::error!("Password hashing error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Hashing Error".to_string())
            }
            AppError::JwtError(e) => {
                tracing::error!("JWT error: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "An internal error occurred".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::SqlxError(err),
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AppError::PasswordHashError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::JwtError(err)
    }
}