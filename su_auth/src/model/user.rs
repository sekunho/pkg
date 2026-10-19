use argon2::PasswordHasher as _;
use deadpool_sqlite::rusqlite::{self, Row, types::FromSql};
use rusqlite::{ToSql, types::ToSqlOutput};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy)]
pub struct UserId(pub(crate) i64);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<UserId> for i64 {
    fn from(val: UserId) -> Self {
        val.0
    }
}

impl FromSql for UserId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let id = value.as_i64()?;
        Ok(UserId(id))
    }
}

impl ToSql for UserId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0))
    }
}

#[derive(Clone)]
pub struct PasswordHash(String);

const DUMMY_USER_PASSWORD_HASH_RAW: &str = "abcd1234";

impl PasswordHash {
    pub fn new(ctx: &argon2::Argon2, password: &Password) -> argon2::password_hash::Result<Self> {
        Ok(Self(ctx.hash_password(password.0.as_bytes())?.to_string()))
    }

    pub fn dummy(ctx: &argon2::Argon2) -> Self {
        Self(
            ctx.hash_password(DUMMY_USER_PASSWORD_HASH_RAW.as_bytes())
                .expect(":D")
                .to_string(),
        )
    }
}

impl std::fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PasswordHash").field(&"[REDACTED]").finish()
    }
}

impl std::fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<PasswordHash> for String {
    fn from(val: PasswordHash) -> Self {
        val.0
    }
}

impl AsRef<str> for PasswordHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromSql for PasswordHash {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_str()?;
        Ok(PasswordHash(value.to_string()))
    }
}

impl ToSql for PasswordHash {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.clone()))
    }
}

pub struct Password(String);

impl Password {
    #[cfg(test)]
    pub(crate) fn dummy() -> Password {
        Password("hunter2".to_string())
    }
}

impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Password> for String {
    fn from(val: Password) -> Self {
        val.0
    }
}

impl<'t> From<&'t Password> for &'t [u8] {
    fn from(val: &'t Password) -> Self {
        val.0.as_bytes()
    }
}

impl From<Password> for Vec<u8> {
    fn from(val: Password) -> Self {
        val.0.into()
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Password").field(&"[REDACTED]").finish()
    }
}

impl std::fmt::Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

#[derive(Clone, Debug)]
pub struct UserInfo {
    pub user_id: UserId,
    pub password_hash: PasswordHash,
    pub created_at: OffsetDateTime,
    pub verified_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
    pub banned_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum User {
    Verified(VerifiedUser),
    Unverified(UnverifiedUser),
    Deleted(DeletedUser),
    Banned(BannedUser),
}

impl User {
    fn info(&self) -> &UserInfo {
        match self {
            User::Verified(VerifiedUser(info))
            | User::Unverified(UnverifiedUser(info))
            | User::Deleted(DeletedUser(info))
            | User::Banned(BannedUser(info)) => info,
        }
    }

    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.info().user_id
    }

    #[must_use]
    pub fn password_hash(&self) -> &PasswordHash {
        &self.info().password_hash
    }

    #[must_use]
    pub fn deleted_at(&self) -> Option<OffsetDateTime> {
        let user = self.info();
        user.deleted_at
    }

    #[must_use]
    pub fn password_hash_str(&self) -> &str {
        self.info().password_hash.as_ref()
    }
}

impl<'t> TryFrom<&Row<'t>> for User {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'t>) -> Result<Self, Self::Error> {
        let user = UserInfo::try_from(row)?;

        let user = match (user.deleted_at, user.banned_at, user.verified_at) {
            (Some(_), _, _) => User::Deleted(DeletedUser(user)),
            (_, Some(_), _) => User::Banned(BannedUser(user)),
            (_, _, Some(_)) => User::Verified(VerifiedUser(user)),
            _ => User::Unverified(UnverifiedUser(user)),
        };
        Ok(user)
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedUser(UserInfo);
#[derive(Clone, Debug)]
pub struct UnverifiedUser(UserInfo);
#[derive(Clone, Debug)]
pub struct DeletedUser(UserInfo);
#[derive(Clone, Debug)]
pub struct BannedUser(UserInfo);

impl VerifiedUser {
    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.0.user_id
    }
}

impl UnverifiedUser {
    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.0.user_id
    }
}

impl From<VerifiedUser> for UserInfo {
    fn from(val: VerifiedUser) -> Self {
        val.0
    }
}

impl From<UnverifiedUser> for UserInfo {
    fn from(val: UnverifiedUser) -> Self {
        val.0
    }
}

impl From<DeletedUser> for UserInfo {
    fn from(val: DeletedUser) -> Self {
        val.0
    }
}

impl From<BannedUser> for UserInfo {
    fn from(val: BannedUser) -> Self {
        val.0
    }
}

impl TryFrom<&Row<'_>> for UserInfo {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let user_id = row.get::<&str, UserId>("user_id")?;
        let password_hash = row.get::<&str, PasswordHash>("password_hash")?;
        let created_at = row.get::<&str, OffsetDateTime>("created_at")?;
        let verified_at = row.get::<&str, Option<OffsetDateTime>>("verified_at")?;
        let deleted_at = row.get::<&str, Option<OffsetDateTime>>("deleted_at")?;
        let banned_at = row.get::<&str, Option<OffsetDateTime>>("banned_at")?;

        Ok(Self {
            user_id,
            password_hash,
            created_at,
            verified_at,
            deleted_at,
            banned_at,
        })
    }
}

pub struct CreateUser {
    pub password_hash: PasswordHash,
    pub verified_at: Option<OffsetDateTime>,
    pub banned_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
}

#[cfg(test)]
mod tests {
    use crate::model::user::{DUMMY_USER_PASSWORD_HASH_RAW, Password};

    #[test]
    fn dummies_are_not_equal() {
        assert_ne!(Password::dummy().0, DUMMY_USER_PASSWORD_HASH_RAW);
    }
}
