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

#[derive(Debug, Serialize)]
pub struct Success<T> {
    data: T,
}

impl<T> Success<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Serialize)]
pub struct Failure {
    cause: String,
}

impl Failure {
    pub fn new<C: Into<String>>(cause: C) -> Self {
        Self {
            cause: cause.into(),
        }
    }
}

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
            // BAD_REQUEST
            Self::QueryRejection(rejection) => (StatusCode::BAD_REQUEST, rejection.body_text()),
            Self::MissingField(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Browser(pantin_browser::Error::ParseUrl(error)) => {
                (StatusCode::BAD_REQUEST, error.to_string())
            },
            // UNPROCESSABLE_ENTITY
            Self::Browser(pantin_browser::Error::Marionette(
                pantin_marionette::Error::Request(pantin_marionette::request::Error::Response(
                    pantin_marionette::response::Error::CommandFailure(_id, failure),
                )),
            )) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("{}: {}", failure.error, failure.message),
            ),
            // INTERNAL_SERVER_ERROR
            Self::Browser(_) | Self::State(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            },
        };

        (status, Json(Failure::new(message))).into_response()
    }
}

pub type Result<T = Response, E = Error> = result::Result<T, E>;

#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Error))]
pub struct Query<T>(pub T);
