-- verification.verify
UPDATE users
SET verified_at = unixepoch()
WHERE user_id = ?1
  AND verified_at IS NULL
  AND EXISTS (
    SELECT 1
    FROM users_verifications
    WHERE users_verifications.user_id = ?1
      AND users_verifications.code = ?2
      AND unixepoch() < users_verifications.expires_at
  )
RETURNING
  user_id,
  password_hash,
  created_at,
  verified_at,
  banned_at,
  deleted_at;
