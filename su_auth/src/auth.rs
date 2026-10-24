use std::sync::Arc;

use crate::{
    AuthnBackend,
    session::{SessionToken, SessionTokenHash},
};

#[derive(Clone, Debug)]
pub struct AuthHandle<Backend: AuthnBackend> {
    pub(crate) session_name: &'static str,
    pub(crate) backend: Backend,
    pub(crate) inner: Arc<Inner<Backend>>,
}

#[derive(Clone, Debug)]
pub struct Inner<B: AuthnBackend> {
    pub(crate) user: Option<B::User>,
}

impl<B: AuthnBackend> AuthHandle<B> {
    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn session_name(&self) -> &str {
        self.session_name
    }

    pub fn current_user(&self) -> impl Future<Output = Option<B::User>> {
        std::future::ready(self.inner.user.clone())
    }

    pub fn get_user(
        &self,
        token: &SessionToken,
    ) -> impl Future<Output = Result<Option<B::User>, B::GetUserError>>
    where
        Self: Sized,
    {
        self.backend
            .get_user_from_session(SessionTokenHash::from(token))
    }
}
