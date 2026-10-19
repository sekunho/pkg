use argon2::PasswordVerifier as _;

use crate::{
    error,
    model::{
        session::{Login, SessionToken, SessionTokenHash},
        user::{Password, PasswordHash, User, UserId},
    },
    session::repository,
};

pub async fn login(
    db: &su_sqlite::handle::Handle,
    ctx: argon2::Argon2<'static>,
    params: Login,
) -> Result<Option<SessionToken>, error::DbError> {
    let mut user = None;

    if let Some(user_id) = params.user_id {
        let conn = db.get_read_conn().await?;
        user = conn
            .interact(move |conn| crate::user::repository::get_user(conn, &user_id))
            .await??;
    }

    let verified = match user {
        Some(ref user) => verify_password(ctx, params.password, user.password_hash().clone()).await,
        _ => verify_password(ctx, params.password, params.dummy_password_hash).await,
    };

    if user.map_or(true, |u| u.deleted_at().is_some()) || verified.is_err() {
        return Ok(None);
    }

    if let Some(user_id) = params.user_id {
        let token = create_session(db, user_id).await?;
        return Ok(Some(token));
    }
    Ok(None)
}

async fn verify_password(
    ctx: argon2::Argon2<'static>,
    password: Password,
    password_hash: PasswordHash,
) -> Result<(), error::VerifyError> {
    tokio::task::spawn_blocking(move || {
        let pbs = Into::<&[u8]>::into(&password);
        let hash = argon2::PasswordHash::new(password_hash.as_ref())
            .map_err(argon2::password_hash::Error::from)?;
        ctx.verify_password(pbs, &hash)?;
        Ok(())
    })
    .await?
}

pub async fn create_session(
    db: &su_sqlite::handle::Handle,
    user_id: UserId,
) -> Result<SessionToken, error::DbError> {
    let conn = db.get_write_conn().await?;
    let token = conn
        .interact(move |conn| repository::create_session(conn, user_id))
        .await??;
    Ok(token)
}

pub async fn logout(
    db: &su_sqlite::handle::Handle,
    token: SessionToken,
) -> Result<(), error::DbError> {
    let hash = SessionTokenHash::from(token);
    let conn = db.get_write_conn().await?;
    conn.interact(move |conn| repository::delete_session(conn, hash))
        .await??;
    Ok(())
}

/// Validates a given session token and returns the associated user.
/// If the user does not exist, it returns Ok(None).
pub async fn validate_session(
    db: &su_sqlite::handle::Handle,
    token: SessionToken,
) -> Result<Option<User>, error::DbError> {
    let hash = SessionTokenHash::from(token);
    let conn = db.get_read_conn().await?;
    let user = conn
        .interact(move |conn| repository::get_session_user(conn, hash))
        .await??;
    Ok(user)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use argon2::{Algorithm, Argon2, Params, Version};
    use time::OffsetDateTime;

    use crate::{
        model::{
            session::Login,
            user::{CreateUser, Password, PasswordHash, User},
        },
        session::service::login,
        user::repository::batch_create_users,
    };

    const TEST_SCHEMA: &str = "
    CREATE TABLE users (
      user_id INTEGER PRIMARY KEY,
      password_hash TEXT NOT NULL,
      created_at INTEGER NOT NULL DEFAULT (unixepoch()),
      verified_at INTEGER,
      banned_at INTEGER,
      deleted_at INTEGER
    ) STRICT;

    CREATE TABLE sessions(
      token_hash BLOB PRIMARY KEY,
      user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
      -- 30 day expiration
      expires_at INTEGER NOT NULL DEFAULT (unixepoch() + 30 * 24 * 60 * 60)
    ) STRICT;

    CREATE INDEX sessions_user_id ON sessions(user_id);
    ";

    const USERS: [(bool, bool, bool); 6] = [
        (true, false, false),
        (false, false, false),
        (true, true, false),
        (true, false, true),
        (true, true, true),
        (true, false, false),
    ];

    fn test_hasher() -> Argon2<'static> {
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(Params::MIN_M_COST, 1, 1, None).unwrap(),
        )
    }

    async fn setup_db(name: &str) -> (su_sqlite::handle::Handle, Vec<User>) {
        let handle =
            su_sqlite::handle::Handle::builder(PathBuf::from(format!("file:/{name}?vfs=memdb")))
                .build()
                .expect("open in memory DB");

        let conn = handle.get_write_conn().await.unwrap();

        let users = conn
            .interact(
                move |conn| -> Result<Vec<User>, deadpool_sqlite::InteractError> {
                    conn.execute_batch(TEST_SCHEMA).unwrap();
                    let now = OffsetDateTime::now_utc();
                    let at = |set: bool| set.then_some(now);

                    let users = USERS
                        .into_iter()
                        .map(|(verified, banned, deleted)| CreateUser {
                            password_hash: PasswordHash::new(&test_hasher(), &Password::dummy())
                                .unwrap(),
                            verified_at: at(verified),
                            banned_at: at(banned),
                            deleted_at: at(deleted),
                        })
                        .collect::<Vec<_>>();

                    let users = batch_create_users(conn, users.as_slice(), 6).unwrap();
                    Ok(users)
                },
            )
            .await
            .unwrap()
            .unwrap();

        (handle, users)
    }

    #[tokio::test]
    async fn assert_successful_logins_except_deleted_accounts() {
        let (db, users) = setup_db("assert_successful_logins_except_deleted_accounts").await;

        for user in users {
            let token = login(
                &db,
                test_hasher(),
                Login {
                    user_id: Some(*user.user_id()),
                    password: Password::dummy(),
                    dummy_password_hash: PasswordHash::dummy(&test_hasher()),
                },
            )
            .await
            .unwrap();

            assert_eq!(token.is_some(), user.deleted_at().is_none(), "{user:#?}");
        }
    }
}
