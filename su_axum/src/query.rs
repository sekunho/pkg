use std::fmt::Display;

use axum_core::{
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use http::{StatusCode, request::Parts};

use crate::{
    deserialize::DeserializeFormUrlEncoded,
    field::{FieldData, FieldErrors, Fields},
};

#[derive(Debug, Clone)]
pub struct Query<T: DeserializeFormUrlEncoded>(pub T);

#[derive(Debug)]
pub enum QueryRejection {
    InvalidQueries(FieldErrors),
}

impl From<FieldErrors> for QueryRejection {
    fn from(err: FieldErrors) -> Self {
        Self::InvalidQueries(err)
    }
}

impl Display for QueryRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryRejection::InvalidQueries(err) => write!(f, "{}", err.to_string()),
        }
    }
}

impl std::error::Error for QueryRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryRejection::InvalidQueries(errors) => Some(errors),
        }
    }
}

impl IntoResponse for QueryRejection {
    fn into_response(self) -> Response {
        match self {
            QueryRejection::InvalidQueries(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response()
            }
        }
    }
}

impl<S, T> FromRequestParts<S> for Query<T>
where
    S: Send + Sync,
    T: DeserializeFormUrlEncoded + Sized,
{
    type Rejection = QueryRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let bytes = parts.uri.query().unwrap_or_default().as_bytes();
        let data = FieldData::from(bytes);
        let fields = Fields::new(&data, FieldErrors::new());
        let query = T::deserialize(fields).map(|query| Query(query))?;
        Ok(query)
    }
}
