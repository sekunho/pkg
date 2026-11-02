# su_auth

An experimental auth library with an unopionated storage backend for user, sessions,
and verification codes.

## Features

- Session and user axum extractors
- `SessionManager` and `SessionRefresh` middlewares
- Optional batch refreshing on independent refresh requests via a simple `tokio`
`mpsc`.

## Design choices

The initial auth implementation was extracted from a private project but was
directly coupled with SQLite3 and several data models but ran into issues in reusing
this as usually the user model varies depending on the use case. Meanwhile, a
basic auth implementation really only needed maybe the user identifier, credentials,
and a couple of timestamps in my case to represent the state that the user is in.
So while messing around with auth options and `axum-login` seemed like a good fit
because it:

- decoupled the data model of users and their sessions.
- decoupled the storage backend of these entities.
- decoupled from DB drivers

But I was also looking for:

- newtypes like `SessionTokenHash` to prevent me from
mixing it up with `PasswordHash` (yeah i know)
- no `serde`
- no `syn` (in non-dev builds)
- no full `axum` crate but `axum-core`
- no `tower` in core implementation
- no `async_trait`
- no `thiserror`

## Inspiration

- `axum-login` - https://github.com/maxcountryman/axum-login
