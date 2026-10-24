# su_auth

An experimental auth library that uses SQLite3 as its storage backend. This aims
to keep the core library as simple as possible.

## Design choices

The initial auth implementation was extracted from a private project but was
directly coupled with SQLite3 and several data models but ran into issues in reusing
this as usually the user model varies depending on the use case. Meanwhile, a
basic auth implementation really only needed maybe the user identifier, password,
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
- no full `axum` crate but `axum-core`
- no `tower` in core implementation
- no `async_trait`
- no `thiserror`

## Inspiration

- `axum-login` - https://github.com/maxcountryman/axum-login
