use deadpool_sqlite::rusqlite;
use rand::TryRng as _;

use crate::{
    model::{
        session::{SessionToken, SessionTokenHash},
        user::{User, UserId},
    },
    session::sql,
};

pub fn create_session(
    conn: &rusqlite::Connection,
    user_id: UserId,
) -> Result<SessionToken, rusqlite::Error> {
    let mut token = [0u8; 32];
    rand::rngs::SysRng
        .try_fill_bytes(&mut token)
        .expect("infallible");
    let token: SessionToken = SessionToken(token.to_vec());
    let hash = SessionTokenHash::from(&token);

    conn.query_one(sql::INSERT_SESSION, (hash, user_id), |_| Ok(()))?;
    Ok(token)
}

pub fn delete_session(
    conn: &rusqlite::Connection,
    hash: SessionTokenHash,
) -> Result<(), rusqlite::Error> {
    conn.execute(sql::DELETE_SESSION, [hash])?;
    Ok(())
}

pub fn get_session_user(
    conn: &rusqlite::Connection,
    hash: SessionTokenHash,
) -> Result<Option<User>, rusqlite::Error> {
    let result = conn.query_row(sql::SELECT_SESSION_TOKEN, [hash], |row| User::try_from(row));

    match result {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}
