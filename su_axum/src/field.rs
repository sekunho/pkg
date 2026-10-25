use std::{borrow::Cow, collections::HashMap, fmt::Display, str::FromStr};

use axum_core::response::IntoResponse;
use http::StatusCode;

pub struct Fields<'t> {
    data: &'t FieldData<'t>,
    errors: FieldErrors,
}

impl<'t> Fields<'t> {
    pub fn new(data: &'t FieldData<'t>, errors: FieldErrors) -> Self {
        Self { data, errors }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.0.is_empty()
    }

    fn record_result<T, E>(
        &mut self,
        k: &'static str,
        v: Result<Option<T>, E>,
        err: &'static str,
        on_absent: Option<FieldError>,
    ) -> Option<T> {
        match (v, on_absent) {
            (Ok(Some(v)), _) => Some(v),
            (Ok(None), None) => None,
            (Ok(None), Some(on_absent)) => {
                self.errors.0.push((k, on_absent));
                None
            }
            (Err(_), _) => {
                self.errors.0.push((k, FieldError::Invalid(err)));
                None
            }
        }
    }

    pub fn required<T>(&mut self, k: &'static str, err: &'static str) -> Option<T>
    where
        T: FromStr,
    {
        self.record_result(k, self.take_one(k), err, Some(FieldError::Missing))
    }

    pub fn required_vec<T>(&mut self, k: &'static str, err: &'static str) -> Option<Vec<T>>
    where
        T: FromStr,
    {
        self.record_result(k, self.take_vec(k), err, Some(FieldError::Missing))
    }

    pub fn optional<T>(&mut self, k: &'static str, err: &'static str) -> Option<T>
    where
        T: FromStr,
    {
        self.record_result(k, self.take_one(k), err, None)
    }

    pub fn optional_vec<T>(&mut self, k: &'static str, err: &'static str) -> Option<Vec<T>>
    where
        T: FromStr,
    {
        self.record_result(k, self.take_vec(k), err, None)
    }

    fn take_one<T>(&self, key: &str) -> Result<Option<T>, T::Err>
    where
        T: FromStr,
    {
        self.data
            .0
            .get(key)
            .and_then(|vals| vals.first())
            .filter(|v| !v.is_empty())
            .map(|val| T::from_str(val))
            .transpose()
    }

    fn take_vec<T>(&self, k: &str) -> Result<Option<Vec<T>>, T::Err>
    where
        T: FromStr,
    {
        self.data
            .0
            .get(k)
            .map(|v| v.iter().filter(|v| !v.is_empty()))
            .map(|i| i.map(|v| T::from_str(v)).collect())
            .transpose()
    }

    pub fn into_error(self) -> FieldErrors {
        self.errors
    }
}

// So as not to burn my eyes every time I refer to this
pub struct FieldData<'t>(HashMap<Cow<'t, str>, Vec<Cow<'t, str>>>);

impl<'t> FieldData<'t> {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn yoink(self) -> HashMap<Cow<'t, str>, Vec<Cow<'t, str>>> {
        self.0
    }
}

impl<'t, B> From<&'t B> for FieldData<'t>
where
    B: AsRef<[u8]> + 't,
{
    fn from(bytes: &'t B) -> Self {
        let f = form_urlencoded::parse(bytes.as_ref());
        f.fold(FieldData::new(), |mut f: FieldData<'t>, (k, v)| {
            match f.0.get_mut(&k) {
                Some(vals) => vals.push(v),
                None => {
                    f.0.insert(k, vec![v]);
                }
            };
            f
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldErrors(pub Vec<(&'static str, FieldError)>);

impl Default for FieldErrors {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl FieldErrors {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl IntoResponse for FieldErrors {
    fn into_response(self) -> axum_core::response::Response {
        (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
    }
}

impl Display for FieldErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.0.iter().fold(String::new(), |acc, (k, v)| {
            format!("{}\n{}: {}", acc, k, v.to_string())
        });
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone)]
pub enum FieldError {
    Missing,
    Invalid(&'static str),
}

impl Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldError::Missing => write!(f, "field is required"),
            FieldError::Invalid(err) => write!(f, "is invalid: {err}"),
        }
    }
}
