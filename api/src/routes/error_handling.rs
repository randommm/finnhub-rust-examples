use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct AppError {
    code: StatusCode,
    internal_message: String,
    user_message: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    user_message: String,
    error_id: String,
}

impl AppError {
    pub fn new(internal_message: impl Into<String>) -> Self {
        Self {
            internal_message: internal_message.into(),
            user_message: None,
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn with_user_message(self, user_message: impl Into<String>) -> Self {
        Self {
            user_message: Some(user_message.into()),
            ..self
        }
    }
    pub fn with_code(self, code: StatusCode) -> Self {
        Self { code, ..self }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let error_id = Uuid::new_v4().to_string();

        let internal_message =
            format!("Error id: {}. Message: {}", error_id, self.internal_message);
        println!("{}. AppError: {}", error_id, internal_message);

        let user_message = self.user_message.unwrap_or("Server error".to_owned());

        let response = ErrorResponse {
            user_message,
            error_id,
        };

        (self.code, Json(response)).into_response()
    }
}

impl From<dotenvy::Error> for AppError {
    fn from(err: dotenvy::Error) -> Self {
        AppError::new(format!("Dotenv error: {:#}", err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::new(format!("Database query error: {:#}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::new(format!("Serde error: {:#}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::new(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::new(err)
    }
}
