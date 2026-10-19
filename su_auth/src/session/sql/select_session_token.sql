-- session.validate
SELECT
  sessions.user_id,
  users.password_hash,
  users.created_at,
  users.verified_at,
  users.banned_at,
  users.deleted_at
FROM sessions
INNER JOIN users
  ON users.user_id = sessions.user_id
  AND users.deleted_at IS NULL
WHERE token_hash = ?
  AND unixepoch() < sessions.expires_at;
