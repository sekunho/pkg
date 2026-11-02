use std::fmt::{Debug, Display};

use crate::AuthnBackend;

#[derive(Debug, Clone)]
pub struct UserId<Backend: AuthnBackend>(
    <<Backend as AuthnBackend>::User as AuthUser<Backend>>::Id,
);

impl<Backend> UserId<Backend>
where
    Backend: AuthnBackend,
{
    pub fn new(user_id: <<Backend as AuthnBackend>::User as AuthUser<Backend>>::Id) -> Self {
        UserId(user_id)
    }

    pub fn raw_id(&self) -> &<<Backend as AuthnBackend>::User as AuthUser<Backend>>::Id {
        &self.0
    }
}

pub trait AuthUser<Backend: AuthnBackend>: Debug + Display + Clone + Send + Sync {
    type Id: Debug + Display + Clone + Send + Sync;

    fn id(&self) -> UserId<Backend>;
}

#[derive(Clone, Debug)]
pub struct CurrentUser<B: AuthnBackend>(pub B::User);
