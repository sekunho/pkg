use std::fmt::{Debug, Display};

use crate::{AuthnBackend, session::SessionTokenHash};

pub struct UserId<Backend: AuthnBackend>(
    <<Backend as AuthnBackend>::User as AuthUser<Backend>>::Id,
);

pub trait AuthUser<Backend: AuthnBackend>: Debug + Display + Clone + Send + Sync {
    type Id: Debug + Display + Clone + Send + Sync;

    fn id(self) -> Self::Id;
    fn session_token_hash(&self) -> &SessionTokenHash;
    fn state(&self) -> &UserState;
}

#[derive(Clone, Debug)]
pub struct CurrentUser<B: AuthnBackend>(pub(crate) B::User);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UserState {
    Verified,
    Suspended,
    Deleted,
}
