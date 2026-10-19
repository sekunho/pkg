use deadpool_sqlite::rusqlite;

use crate::{
    model::{
        user::{User, UserId},
        verification::{Code, CreateCode, Verification},
    },
    verification::sql,
};

pub fn upsert_code(
    conn: &rusqlite::Connection,
    params: &CreateCode,
) -> Result<Option<Verification>, rusqlite::Error> {
    let result = conn.query_row(
        sql::INSERT_CODE,
        (params.user_id, &params.code, params.ttl.whole_seconds()),
        |row| Verification::try_from(row),
    );

    match result {
        Ok(code) => Ok(Some(code)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn get_current_code(
    conn: &rusqlite::Connection,
    user_id: UserId,
) -> Result<Option<Verification>, rusqlite::Error> {
    let result = conn.query_row(sql::SELECT_CODE, [user_id], |row| {
        Verification::try_from(row)
    });

    match result {
        Ok(code) => Ok(Some(code)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn delete_codes(conn: &rusqlite::Connection, user_id: UserId) -> Result<(), rusqlite::Error> {
    conn.execute(sql::DELETE_CODE, [user_id])?;
    Ok(())
}

pub fn verify(
    conn: &rusqlite::Connection,
    user_id: UserId,
    code: &Code,
) -> Result<Option<User>, rusqlite::Error> {
    let result = conn.query_row(sql::VERIFY_USER, (user_id, code), |row| User::try_from(row));

    let user = match result {
        Ok(user) => user,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(err) => return Err(err),
    };

    delete_codes(conn, user_id)?;
    Ok(Some(user))
}
