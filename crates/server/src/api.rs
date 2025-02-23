use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::eyre;

pub struct ServerError(eyre::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E: Into<eyre::Error>> From<E> for ServerError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn ping() -> Result<impl IntoResponse, ServerError> {
    Ok("pong")
}
