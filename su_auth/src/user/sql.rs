pub(crate) const SELECT_USER_BY_ID: &str = include_str!("sql/select_user_by_id.sql");

#[must_use]
pub(crate) fn insert_users(rows: usize, columns: usize) -> String {
    let rows = su_sqlite::query::generate_value_rows(rows, columns);
    format!(include_str!("sql/insert_users.sql"), rows = rows)
}

#[must_use]
pub(crate) fn delete_users(ids: usize) -> String {
    let ids = su_sqlite::query::generate_value_rows(1, ids);
    format!(include_str!("sql/delete_users.sql"), ids = ids)
}
