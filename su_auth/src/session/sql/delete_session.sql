-- session.delete
DELETE FROM sessions
WHERE token_hash = ?;
