-- user.batch_create
INSERT INTO users(password_hash, verified_at, banned_at, deleted_at)
VALUES {rows}
RETURNING
  user_id,
  password_hash,
  created_at,
  verified_at,
  banned_at,
  deleted_at;
