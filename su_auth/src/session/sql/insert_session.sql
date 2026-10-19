-- session.create
INSERT INTO sessions (token_hash, user_id)
VALUES (?, ?)
RETURNING user_id;
