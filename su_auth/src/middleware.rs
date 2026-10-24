use std::pin::Pin;

use axum_core::extract::Request;
use tower_layer::Layer;
use tower_service::Service;

use crate::{AuthnBackend, auth::AuthHandle};

pub struct AuthSessionRefreshLayer<Backend: AuthnBackend> {
    pub session: AuthHandle<Backend>,
}

impl<S, Backend> Layer<S> for AuthSessionRefreshLayer<Backend>
where
    Backend: AuthnBackend + Send + Sync,
{
    type Service = AuthSessionRefreshService<S, Backend>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            session: self.session.clone(),
        }
    }
}

pub struct AuthSessionRefreshService<S, Backend: AuthnBackend> {
    pub inner: S,
    pub session: AuthHandle<Backend>,
}

impl<S, B, Backend> Service<Request<B>> for AuthSessionRefreshService<S, Backend>
where
    Backend: AuthnBackend + Send + Sync + 'static,
    S: Service<Request<B>> + Clone + 'static,
    B: 'static
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let session = self.session.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let _ = session.refresh_session().await;
            inner.call(req).await
        })
    }
}
