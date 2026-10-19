use deadpool_sqlite::rusqlite::{self, types::Value};

use crate::{
    model::user::{CreateUser, User, UserId},
    user::sql,
};

pub fn batch_create_users(
    conn: &rusqlite::Connection,
    params: &[CreateUser],
    chunk_size: usize,
) -> Result<Vec<User>, rusqlite::Error> {
    let chunk_size = if chunk_size == 0 { 1 } else { chunk_size };
    let mut users = Vec::with_capacity(params.len());

    for chunked_params in params.chunks(chunk_size) {
        let chunked_values = rusqlite::params_from_iter(chunked_params.iter().flat_map(|param| {
            [
                Value::Text(param.password_hash.as_ref().to_owned()),
                param
                    .verified_at
                    .map_or(Value::Null, |t| Value::Integer(t.unix_timestamp())),
                param
                    .banned_at
                    .map_or(Value::Null, |t| Value::Integer(t.unix_timestamp())),
                param
                    .deleted_at
                    .map_or(Value::Null, |t| Value::Integer(t.unix_timestamp())),
            ]
        }));

        // Don't forget to update 4 into the # of columns!!!
        let mut stmt = conn.prepare(sql::insert_users(chunked_params.len(), 4).as_ref())?;
        let rows = stmt.query_map(chunked_values, |row| User::try_from(row))?;
        for user in rows {
            users.push(user?);
        }
    }

    Ok(users)
}

pub fn get_user(
    conn: &rusqlite::Connection,
    user_id: &UserId,
) -> Result<Option<User>, rusqlite::Error> {
    let result = conn.query_row(sql::SELECT_USER_BY_ID, [user_id.0], |row| {
        User::try_from(row)
    });

    match result {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn batch_hard_delete_users(
    conn: &rusqlite::Connection,
    user_ids: &[UserId],
    chunk_size: usize,
) -> Result<Vec<User>, rusqlite::Error> {
    let chunk_size = if chunk_size == 0 { 1 } else { chunk_size };
    let mut users = Vec::with_capacity(user_ids.len());

    for chunked_ids in user_ids.chunks(chunk_size) {
        let mut stmt = conn.prepare(sql::delete_users(chunked_ids.len()).as_ref())?;
        let rows = stmt.query_map(rusqlite::params_from_iter(chunked_ids), |row| {
            User::try_from(row)
        })?;
        for user in rows {
            users.push(user?);
        }
    }

    Ok(users)
}
