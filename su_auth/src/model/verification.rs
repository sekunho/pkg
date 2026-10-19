use deadpool_sqlite::rusqlite::{
    self, Row,
    types::{FromSql, ToSql, ToSqlOutput},
};
use time::OffsetDateTime;

use crate::model::user::UserId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code(String);

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

impl FromSql for Code {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_str()?;
        Ok(Code(value.to_string()))
    }
}

impl ToSql for Code {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.as_str()))
    }
}

#[derive(Clone, Debug)]
pub struct Verification {
    pub user_id: UserId,
    pub code: Code,
    pub expires_at: OffsetDateTime,
}

impl TryFrom<&Row<'_>> for Verification {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let user_id = row.get::<&str, UserId>("user_id")?;
        let code = row.get::<&str, Code>("code")?;
        let expires_at = row.get::<&str, OffsetDateTime>("expires_at")?;

        Ok(Self {
            user_id,
            code,
            expires_at,
        })
    }
}

pub struct CreateCode {
    pub user_id: UserId,
    pub code: Code,
    pub ttl: time::Duration,
}
