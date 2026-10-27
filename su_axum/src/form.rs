use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use axum_core::{
    extract::{FromRequest, Request, rejection::BytesRejection},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use http::{StatusCode, header};

use crate::{
    deserialize::DeserializeFormUrlEncoded,
    field::{FieldData, FieldErrors, Fields},
};

pub const EXPECTED_FORM_CONTENT_TYPE: &str =
    "Expected request with Content-Type: application/x-www-form-urlencoded";

#[derive(Debug, Clone)]
pub struct Form<T: DeserializeFormUrlEncoded>(pub T);

impl<S, T> FromRequest<S> for Form<T>
where
    S: Send + Sync,
    T: DeserializeFormUrlEncoded + Sized,
{
    type Rejection = FormRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        req.headers()
            .get(header::CONTENT_TYPE)
            .ok_or(FormRejection::InvalidFormContentType)
            .and_then(|v| {
                v.to_str()
                    .map_err(|_| FormRejection::InvalidFormContentType)
            })
            .and_then(|v| Ok(v.starts_with(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref())))
            .and_then(|valid| {
                if valid {
                    Ok(())
                } else {
                    Err(FormRejection::InvalidFormContentType)
                }
            })?;

        let bytes = Bytes::from_request(req, state).await?;
        let data = FieldData::from(bytes.as_ref());
        let fields = Fields::new(&data, FieldErrors::new());

        match T::deserialize(fields) {
            Ok(form) => Ok(Form(form)),
            Err(errors) => {
                let raw_form = data
                    .yoink()
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k.into_owned(),
                            v.into_iter().map(|v| v.into_owned()).collect(),
                        )
                    })
                    .collect();

                Err(FormRejection::InvalidBody {
                    errors,
                    raw_form,
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum FormRejection {
    Bytes(BytesRejection),
    InvalidFormContentType,
    InvalidBody {
        errors: FieldErrors,
        raw_form: HashMap<String, Vec<String>>,
    },
}

impl Display for FormRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormRejection::Bytes(err) => write!(f, "{}", err.to_string()),
            FormRejection::InvalidFormContentType => write!(f, "{}", EXPECTED_FORM_CONTENT_TYPE),
            FormRejection::InvalidBody { errors: err, .. } => write!(f, "{}", err.to_string()),
        }
    }
}

impl std::error::Error for FormRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FormRejection::Bytes(err) => Some(err),
            FormRejection::InvalidFormContentType => None,
            FormRejection::InvalidBody { errors, .. } => Some(errors),
        }
    }
}

impl From<BytesRejection> for FormRejection {
    fn from(err: BytesRejection) -> Self {
        FormRejection::Bytes(err)
    }
}

impl IntoResponse for FormRejection {
    fn into_response(self) -> Response {
        match self {
            FormRejection::Bytes(err) => err.into_response(),
            FormRejection::InvalidFormContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                EXPECTED_FORM_CONTENT_TYPE,
            )
                .into_response(),
            FormRejection::InvalidBody { errors: err, .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, err.into_response()).into_response()
            }
        }
    }
}
