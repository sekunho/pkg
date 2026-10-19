-- verification.get
SELECT
  user_id,
  code,
  expires_at
FROM users_verifications
WHERE user_id = ?
  AND unixepoch() < expires_at;
