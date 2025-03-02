//! This module provides helper types and error handling for API responses in pantin server.
//!
//! It defines response wrappers for both success and error cases, as well as a custom error type that
//! aggregates errors from various parts of the system (state, browser operations, and query extraction).
//!
//! The key types defined here are:
//!
//! - [`Success<T>`]: A generic wrapper for successful responses. It holds data of type `T`.
//! - [`Failure`]: A wrapper for error responses. It contains an error message describing the cause.
//! - [`Error`]: An enumeration of errors that can occur within the server, including errors from state handling,
//!   browser operations, and query extraction. It implements [`IntoResponse`] so that errors are automatically
//!   converted into HTTP responses with appropriate status codes and JSON bodies.
//! - [`Query<T>`]: A wrapper for extracting query parameters from HTTP request parts using Axum.
//!
//! # Usage
//!
//! When building API endpoints, you can use [`Success<T>`] to wrap successful responses and propagate errors
//! that will be converted into JSON responses automatically by Axum's response system.
//!
//! ## Example
//!
//! ```rust
//! use axum::Json;
//! use pantin_server::api::{Success, Error, Query};
//!
//! // Assume MyQuery and MyData are defined elsewhere.
//! async fn get_data(query: Query<MyQuery>) -> Result<Json<Success<MyData>>, Error> {
//!     // Process the query and fetch data...
//!     let data = fetch_data(query.0).await?;
//!     Ok(Json(Success::new(data)))
//! }
//! ```
//!
//! # Error Handling
//!
//! The [`Error`] enum converts errors from state management, browser operations, and query extraction into
//! HTTP responses. Depending on the error variant, it returns appropriate HTTP status codes such as 400 (Bad Request),
//! 422 (Unprocessable Entity), or 500 (Internal Server Error) along with a JSON error message.
//!
//! Internally, the error is logged using the `tracing` crate before being transformed into a response.

use std::result;

use axum::{
    Json,
    extract::{FromRequestParts, rejection::QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

use crate::state;

/// A generic wrapper for successful API responses.
///
/// This type encapsulates the response data, which is then serialized as JSON.
/// The field is named `data` in the JSON output.
#[derive(Debug, Serialize)]
pub struct Success<T> {
    data: T,
}

impl<T> Success<T> {
    /// Creates a new [`Success`] instance wrapping the given data.
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

/// A wrapper for error responses.
///
/// This type encapsulates an error message that will be sent back as JSON.
/// The message field is named `cause` in the JSON output.
#[derive(Debug, Serialize)]
pub struct Failure {
    cause: String,
}

impl Failure {
    /// Creates a new [`Failure`] instance with the specified error cause.
    pub fn new<C: Into<String>>(cause: C) -> Self {
        Self {
            cause: cause.into(),
        }
    }
}

/// An enumeration of errors that can occur in Pantin Server.
///
/// This enum aggregates errors from state management, browser operations, and query extraction.
/// It implements [`IntoResponse`] so that errors are automatically converted into HTTP responses.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    State(#[from] state::Error),
    #[error(transparent)]
    Browser(#[from] pantin_browser::Error),
    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),
    #[error("missing field: {0}")]
    MissingField(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{:?}", self);

        let (status, message) = match self {
            // Return `BAD_REQUEST` for query extraction errors, missing fields or URL parsing errors.
            Self::QueryRejection(rejection) => (StatusCode::BAD_REQUEST, rejection.body_text()),
            Self::MissingField(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Browser(pantin_browser::Error::ParseUrl(error)) => {
                (StatusCode::BAD_REQUEST, error.to_string())
            },
            // Return `UNPROCESSABLE_ENTITY` for command failures.
            Self::Browser(pantin_browser::Error::Marionette(
                pantin_marionette::Error::Request(pantin_marionette::request::Error::Response(
                    pantin_marionette::response::Error::CommandFailure(_id, failure),
                )),
            )) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("{}: {}", failure.error, failure.message),
            ),
            // All other errors result in `INTERNAL_SERVER_ERROR`.
            Self::Browser(_) | Self::State(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            },
        };

        (status, Json(Failure::new(message))).into_response()
    }
}

/// A specialized result type for API response.
pub type Result<T = Response, E = Error> = result::Result<T, E>;

/// A wrapper type for extracting query parameters from HTTP request parts.
///
/// This type uses Axum's [`FromRequestParts`] to extract query parameters via [`axum::extract::Query`]
/// and converts any rejection into an [`Error`] with appropriate HTTP status code.
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Error))]
pub struct Query<T>(pub T);

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_success_serialization() {
        let success = Success::new("ok");
        let json = serde_json::to_string(&success).expect("Serialization should succeed");
        assert!(json.contains(r#""data":"ok""#));
    }

    #[test]
    fn test_failure_serialization() {
        let failure = Failure::new("error");
        let json = serde_json::to_string(&failure).expect("Serialization should succeed");
        assert!(json.contains(r#""cause":"error""#));
    }

    #[tokio::test]
    async fn test_error_into_response() {
        let error = Error::MissingField("field".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let error = Error::Browser(pantin_browser::Error::ParseUrl(url::ParseError::EmptyHost));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let error = Error::Browser(pantin_browser::Error::Marionette(
            pantin_marionette::Error::Request(pantin_marionette::request::Error::Response(
                pantin_marionette::response::Error::CommandFailure(
                    42,
                    pantin_marionette::response::Failure {
                        error: "test-error".into(),
                        message: "test-message".into(),
                        stacktrace: "test-trace".into(),
                    },
                ),
            )),
        ));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        // TODO: add more testes
    }
}
