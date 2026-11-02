use std::{error::Error, marker::PhantomData, pin::Pin, sync::Arc};

use axum_core::{
    extract::Request,
    response::{IntoResponse as _, Response},
};
use axum_extra::extract::CookieJar;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use tokio::sync::Mutex;
use tower_layer::Layer;
use tower_service::Service;

use crate::{
    AuthnBackend,
    auth::{AuthHandle, SessionData},
    cookie,
    error::AuthRejection,
    session::{AuthSession as _, SessionToken, SessionTokenHash},
    user::CurrentUser,
};

#[derive(Clone)]
pub struct SessionManagerLayer<Backend: AuthnBackend> {
    pub backend: Backend,
    /// This will be used as the session cookie key
    pub session_name: String,

    /// The max duration a session is allowed to be extended to.
    pub max_session_duration: std::time::Duration,

    /// A session may only valid for the maximum of this duration. If a session
    /// was created 90 days ago, and the absolute is only 60 days, then the session
    /// is invalid and will be thrown away. The inner session of the `AuthHandle`
    /// will be `None`.
    ///
    /// After which the user must re-authenticate again.
    pub max_session_lifetime: std::time::Duration,

    pub refresh_buffer: Option<tokio::sync::mpsc::Sender<(SessionTokenHash, Backend::Session)>>,
}

impl<S, Backend> Layer<S> for SessionManagerLayer<Backend>
where
    Backend: AuthnBackend + Send + Sync,
{
    type Service = SessionManagerService<S, Backend>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            backend: self.backend.clone(),
            session_name: self.session_name.clone(),
            max_session_lifetime: self.max_session_lifetime,
            max_session_duration: self.max_session_duration,
            refresh_buffer: self.refresh_buffer.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SessionManagerService<S, Backend: AuthnBackend> {
    pub(crate) inner: S,
    pub(crate) session_name: String,
    pub(crate) backend: Backend,
    pub(crate) refresh_buffer:
        Option<tokio::sync::mpsc::Sender<(SessionTokenHash, Backend::Session)>>,
    pub(crate) max_session_duration: std::time::Duration,
    pub(crate) max_session_lifetime: std::time::Duration,
}

impl<S, Backend> SessionManagerService<S, Backend>
where
    Backend: AuthnBackend,
{
    async fn validate_token(
        backend: &Backend,
        token: Option<SessionToken>,
        lifetime: std::time::Duration,
    ) -> Result<Option<SessionData<Backend>>, Box<dyn Error + Send + Sync>> {
        let Some(token) = token else { return Ok(None) };
        let hash = token.clone().into();
        let Some(s) = backend.validate_session(hash).await? else {
            return Ok(None);
        };
        if s.expires_in().is_zero() || s.created_ago() > lifetime {
            return Ok(None);
        }

        let Some(user) = backend.get_user_from_session(hash).await? else {
            return Ok(None);
        };

        Ok(Some(SessionData {
            session: s,
            token,
            user: CurrentUser(user),
        }))
    }
}

impl<S, B, Backend> Service<Request<B>> for SessionManagerService<S, Backend>
where
    Backend: AuthnBackend + Send + Sync + 'static,
    S: Service<Request<B>, Response = Response> + Clone + 'static + Send,
    S::Future: Send,
    B: 'static + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        // NOTE: Reminder to myself not to use the cloned inner directly!
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let session_name = self.session_name.clone();
        let backend = self.backend.clone();
        let jar = CookieJar::from_headers(req.headers());
        let max_session_lifetime = self.max_session_lifetime;
        let max_session_duration = self.max_session_duration;
        let refresh_buffer = self.refresh_buffer.clone();

        let token = jar
            .get(&self.session_name)
            .map(|c| URL_SAFE_NO_PAD.decode(c.value()).map(SessionToken))
            .transpose()
            .inspect_err(|e| tracing::error!("{}", e.to_string()))
            .unwrap_or(None);

        Box::pin(async move {
            let data = match Self::validate_token(&backend, token, max_session_lifetime).await {
                Ok(data) => data,
                Err(err) => {
                    tracing::error!("failed to validate token: {}", err);
                    return Ok(AuthRejection::internal().into_response());
                }
            };

            let handle = AuthHandle {
                session_name,
                backend,
                session_data: Arc::new(Mutex::new(data)),
                max_session_duration,
                max_session_lifetime,
                refresh_buffer,
            };

            req.extensions_mut().insert::<AuthHandle<Backend>>(handle);
            inner.call(req).await
        })
    }
}

#[derive(Clone)]
pub struct SessionRefreshLayer<Backend: AuthnBackend> {
    /// This will be used as the session cookie key
    pub session_name: String,
    pub phantom: PhantomData<Backend>,

    /// For every refresh attempt, the session must have a specific duration left
    /// before the next refresh attempt.
    pub refresh_throttle: std::time::Duration,
}

impl<S, Backend> Layer<S> for SessionRefreshLayer<Backend>
where
    Backend: AuthnBackend + Send + Sync,
{
    type Service = SessionRefreshService<S, Backend>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            session_name: self.session_name.clone(),
            phantom: PhantomData,
            refresh_throttle: self.refresh_throttle,
        }
    }
}

#[derive(Clone)]
pub struct SessionRefreshService<S, Backend: AuthnBackend> {
    pub inner: S,
    pub session_name: String,
    phantom: PhantomData<Backend>,
    pub refresh_throttle: std::time::Duration,
}

impl<S, B, Backend> Service<Request<B>> for SessionRefreshService<S, Backend>
where
    Backend: AuthnBackend + Send + Sync + 'static,
    S: Service<Request<B>, Response = Response> + Clone + 'static + Send,
    S::Future: Send,
    B: 'static + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let c_key = self.session_name.clone();
        let Some(auth) = req.extensions().get::<AuthHandle<Backend>>().cloned() else {
            return Box::pin(inner.call(req));
        };

        Box::pin(async move {
            let _ = auth
                .refresh_session()
                .await
                .inspect_err(|e| tracing::error!("{}", e));

            let mut res = inner.call(req).await?;

            let cookie = match auth.session_data.lock().await.as_ref() {
                Some(data) => {
                    let d = time::Duration::try_from(data.session.expires_in())
                        .unwrap_or(time::Duration::MAX);
                    cookie::create_session_cookie(c_key, data.token.clone(), d)
                }
                None => cookie::invalidate(c_key.as_str()),
            };

            cookie::set_cookie_header(res.headers_mut(), cookie);
            Ok(res)
        })
    }
}
