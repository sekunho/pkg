use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    AuthnBackend,
    error::{LoginError, LogoutError, RefreshError},
    session::{AuthSession as _, SessionToken, SessionTokenHash},
    user::{AuthUser as _, CurrentUser, UserId},
};

#[derive(Clone, Debug)]
pub struct SessionData<Backend: AuthnBackend> {
    pub session: Backend::Session,
    pub token: SessionToken,
    pub user: CurrentUser<Backend>,
}

#[derive(Clone, Debug)]
pub struct AuthHandle<Backend: AuthnBackend> {
    pub(crate) session_name: String,
    pub(crate) backend: Backend,
    pub(crate) session_data: Arc<Mutex<Option<SessionData<Backend>>>>,

    /// The max duration a session is allowed to be extended to.
    pub max_session_duration: std::time::Duration,

    /// A session may only valid for the maximum of this duration. If a session
    /// was created 90 days ago, and the absolute is only 60 days, then the session
    /// is invalid.
    ///
    /// If the session is only 56 days old, then the next refresh will only extend
    /// up to 4 more days.
    ///
    /// After which the user must re-authenticate again.
    pub max_session_lifetime: std::time::Duration,

    pub refresh_buffer: Option<tokio::sync::mpsc::Sender<(SessionTokenHash, Backend::Session)>>,
}

impl<B: AuthnBackend> AuthHandle<B> {
    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn session_name(&self) -> &str {
        self.session_name.as_ref()
    }

    pub async fn session_expires_in(&self) -> Option<std::time::Duration> {
        self.session_data
            .lock()
            .await
            .as_ref()
            .map(|s| s.session.expires_in())
    }

    pub async fn session_created_ago(&self) -> Option<std::time::Duration> {
        self.session_data
            .lock()
            .await
            .as_ref()
            .map(|s| s.session.created_ago())
    }

    pub async fn session_token_hash(&self) -> Option<SessionTokenHash> {
        self.session_data
            .lock()
            .await
            .as_ref()
            .map(|s| s.session.token_hash())
    }

    pub async fn session_user_id(&self) -> Option<UserId<B>> {
        self.session_data
            .lock()
            .await
            .as_ref()
            .map(|s| s.session.user_id())
    }

    pub async fn login(&self, creds: B::Credentials) -> Result<(), LoginError<B>> {
        if self.session_data.lock().await.as_ref().is_some() {
            return Err(LoginError::AlreadyLoggedIn);
        }

        let user = self
            .backend()
            .authenticate(creds)
            .await
            .map_err(LoginError::Authenticate)?;

        let token = B::generate_session_token();
        let token_hash = SessionTokenHash::from(token.clone());
        let current_session = B::Session::new(user.id(), token_hash);

        let current_session = self
            .backend()
            .create_session(current_session)
            .await
            .map_err(LoginError::CreateSession)?;

        let mut session_data = self.session_data.lock().await;
        *session_data = Some(SessionData {
            session: current_session,
            token,
            user: CurrentUser(user),
        });
        Ok(())
    }

    pub async fn logout(&self) -> Result<(), LogoutError<B>> {
        let Some(token_hash) = self.session_token_hash().await else {
            return Err(LogoutError::NoActiveSession);
        };

        self.backend()
            .delete_session(token_hash)
            .await
            .map_err(LogoutError::DeleteSession)?;

        self.clear_session_data().await;
        Ok(())
    }

    pub async fn logout_all(&self) -> Result<(), LogoutError<B>> {
        let Some(user_id) = self.session_user_id().await else {
            return Err(LogoutError::NoActiveSession);
        };

        self.backend()
            .delete_all_session(&user_id)
            .await
            .map_err(LogoutError::DeleteSession)?;

        self.clear_session_data().await;
        Ok(())
    }

    pub(crate) async fn refresh_session(&self) -> Result<(), RefreshError<B>> {
        let Some(created_ago) = self.session_created_ago().await else {
            return Ok(());
        };

        let Some(token_hash) = self.session_token_hash().await else {
            return Ok(());
        };

        let remaining = self.max_session_lifetime.saturating_sub(created_ago);
        let new_duration = self.max_session_duration.min(remaining);
        if new_duration.is_zero() {
            self.clear_session_data().await;
            return Ok(());
        }

        let Some(session) = self
            .session_data
            .lock()
            .await
            .as_ref()
            .map(|s| s.session.clone())
        else {
            return Ok(());
        };

        let count = match self.refresh_buffer {
            Some(ref chan) => chan.try_send((token_hash, session.clone())).map(|_| 1)?,
            None => self
                .backend
                .update_sessions(&[session])
                .await
                .map_err(RefreshError::Update)?,
        };

        // I think it's ok if the session is extended despite the queue path
        // not being a guaranteed write as the backend is still checked at the
        // start of each request.
        if count == 1 {
            self.set_session_duration(new_duration).await;
            Ok(())
        } else {
            Err(RefreshError::MissingUpdate)
        }
    }

    /// Sets the current session data to `None`.
    pub(crate) async fn clear_session_data(&self) {
        let mut guard = self.session_data.lock().await;
        *guard = None;
    }

    /// Sets the auth handle's session's duration if it exists.
    pub(crate) async fn set_session_duration(&self, duration: std::time::Duration) {
        let mut guard = self.session_data.lock().await;
        let Some(session_data) = guard.as_mut() else {
            return;
        };

        session_data.session.extend_expiration(duration);
    }
}
