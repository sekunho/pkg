# Database models

## Users

Only these core user info are needed. You can either extend this by creating a new table with a reference to the `users` table, or extend the `users` table. Provided that the columns don't get touched.

```sql
CREATE TABLE users (
  user_id INTEGER PRIMARY KEY,
  password_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  verified_at INTEGER,
  banned_at INTEGER,
  deleted_at INTEGER
) STRICT;
```

## Sessions

Sessions and user session token hashes are persisted in the table. You're free to choose your own expiration time.

```sql
CREATE TABLE sessions(
  token_hash BLOB PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,

  -- 30 day expiration
  expires_at INTEGER NOT NULL DEFAULT (unixepoch() + 30 * 24 * 60 * 60)
) STRICT;
```

## Account verification

```sql
CREATE TABLE users_verifications (
  user_id INTEGER PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
  code TEXT NOT NULL,
  expires_at INTEGER NOT NULL DEFAULT (unixepoch() + 15 * 60)
) STRICT;
```
