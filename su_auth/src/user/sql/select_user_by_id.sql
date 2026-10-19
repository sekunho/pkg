-- user.get
SELECT
  user_id,
  password_hash,
  created_at,
  verified_at,
  banned_at,
  deleted_at
FROM users
WHERE users.user_id = ?;
