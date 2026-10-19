-- user.batch_delete
DELETE FROM users
WHERE user_id IN {ids}
RETURNING
  user_id,
  password_hash,
  created_at,
  verified_at,
  banned_at,
  deleted_at;
