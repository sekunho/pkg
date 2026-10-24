use crate::{AuthnBackend, user::UserId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code(String);

pub trait AuthVerification<Backend: AuthnBackend> {
    fn code(&self) -> Code;
    fn user_id(&self) -> UserId<Backend>;
}

impl From<String> for Code {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Code> for String {
    fn from(val: Code) -> Self {
        val.0
    }
}

impl AsRef<str> for Code {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "rusqlite")]
impl rusqlite::types::FromSql for Code {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_str()?;
        Ok(Code(value.to_string()))
    }
}

#[cfg(feature = "rusqlite")]
impl rusqlite::types::ToSql for Code {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.0.as_str()))
    }
}
