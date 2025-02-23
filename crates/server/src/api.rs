use axum::{
    extract::FromRequestParts,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use color_eyre::eyre;
use serde::Serialize;
use tracing::error;

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
    code: String,
    message: String,
}

impl Failure {
    pub const fn new(code: String, message: String) -> Self {
        Self { code, message }
    }
}

pub struct Error(eyre::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Failure::new(
                "INTERNAL_SERVER_ERROR".into(),
                format!("{}", self.0),
            )),
        )
            .into_response()
    }
}

impl<E: Into<eyre::Error>> From<E> for Error {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Error))]
pub struct Query<T>(pub T);
