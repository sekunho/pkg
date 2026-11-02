use std::collections::HashMap;

use tokio::sync::mpsc::Receiver;

use crate::{AuthnBackend, session::SessionTokenHash};

pub struct SessionRefreshWorker<Backend: AuthnBackend> {
    pub(crate) backend: Backend,
    pub(crate) rx: Receiver<(SessionTokenHash, Backend::Session)>,
    pub flush_interval: std::time::Duration,
    pub batch_size: usize,
}

impl<Backend> SessionRefreshWorker<Backend>
where
    Backend: AuthnBackend,
{
    pub fn new(backend: Backend, rx: Receiver<(SessionTokenHash, Backend::Session)>) -> Self {
        Self {
            backend,
            batch_size: 1,
            rx,
            flush_interval: std::time::Duration::default(),
        }
    }

    pub async fn run(mut self) {
        let mut buffer: Vec<(SessionTokenHash, Backend::Session)> =
            Vec::with_capacity(self.batch_size);
        while let count = self.rx.recv_many(&mut buffer, self.batch_size).await
            && count > 0
        {
            let buffer: Vec<_> = buffer
                .drain(..)
                .collect::<HashMap<SessionTokenHash, _>>()
                .into_values()
                .collect();
            let actual_count = self
                .backend
                .update_sessions(buffer.as_ref())
                .await
                .inspect_err(|e| tracing::error!("failed to update session: {}", e.to_string()))
                .unwrap_or(0);
            tracing::debug!("Refreshed {}/{}", actual_count, buffer.len());
            tokio::time::sleep(self.flush_interval).await;
        }
    }
}
