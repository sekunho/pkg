use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    AuthnBackend,
    session::{AuthSession as _, SessionToken, SessionTokenHash},
};

#[derive(Clone, Debug)]
pub struct AuthHandle<Backend: AuthnBackend> {
    pub(crate) session_name: &'static str,
    pub(crate) backend: Backend,
    pub(crate) inner: Arc<Mutex<Inner<Backend>>>,
}

#[derive(Clone, Debug)]
pub struct Inner<B: AuthnBackend> {
    pub(crate) user: Option<B::User>,
    pub(crate) session: Option<B::Session>,
}

impl<B: AuthnBackend> AuthHandle<B> {
    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn session_name(&self) -> &str {
        self.session_name
    }

    pub async fn current_user(&self) -> Option<B::User> {
        self.inner.lock().await.clone().user
    }

    pub async fn refresh_session(&self) -> Result<(), B::UpdateSessionError> {
        let mut inner = self.inner.lock().await;
        let token_hash = inner.session.as_mut().map(|session| session.token_hash());

        if let Some(token_hash) =  token_hash {
            inner.session = self.backend.refresh_session(&token_hash).await?;
        }
        Ok(())
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
