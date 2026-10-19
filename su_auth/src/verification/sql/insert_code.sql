-- verification.create
INSERT INTO users_verifications (user_id, code, expires_at)
VALUES (?, ?, unixepoch() + ?)
ON CONFLICT (user_id) DO UPDATE
  SET code = excluded.code,
      expires_at = excluded.expires_at
  WHERE users_verifications.expires_at <= unixepoch()
RETURNING
  user_id,
  code,
  expires_at;
