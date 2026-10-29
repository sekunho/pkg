use std::{pin::Pin, sync::Arc};

use axum_core::extract::Request;
use tokio::sync::Mutex;
use tower_layer::Layer;
use tower_service::Service;

use crate::{
    AuthnBackend,
    auth::{AuthHandle, SessionData},
};

#[derive(Clone)]
pub struct SessionManagerLayer<Backend: AuthnBackend> {
    pub backend: Backend,
    // This will be used as the session cookie key
    pub session_name: String,
}

impl<S, Backend> Layer<S> for SessionManagerLayer<Backend>
where
    Backend: AuthnBackend + Send + Sync,
{
    type Service = SessionManagerService<S, Backend>;

    fn layer(&self, inner: S) -> Self::Service {
        let backend = self.backend.clone();
        let session_name = self.session_name.clone();

        Self::Service {
            inner,
            backend,
            session_name,
        }
    }
}

#[derive(Clone)]
pub struct SessionManagerService<S, Backend: AuthnBackend> {
    pub inner: S,
    pub session_name: String,
    pub backend: Backend,
}

impl<S, B, Backend> Service<Request<B>> for SessionManagerService<S, Backend>
where
    Backend: AuthnBackend + Send + Sync + 'static,
    S: Service<Request<B>> + Clone + 'static + Send,
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

        Box::pin(async move {
            let session_data = Arc::new(Mutex::from(SessionData {
                user: None,
                session: None,
            }));

            let session_manager = AuthHandle {
                session_name,
                backend,
                session_data,
            };

            req.extensions_mut().insert::<AuthHandle<Backend>>(session_manager);
            inner.call(req).await
        })
    }
}
