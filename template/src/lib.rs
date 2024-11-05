#![forbid(unsafe_code)]
use std::{path::PathBuf, sync::Arc};

use minijinja::Environment;
use serde::Serialize;

pub use minijinja::{context, Error};
use thiserror::Error;

pub struct HandleBuilder {
    env: Environment<'static>,
    watch_paths: Vec<PathBuf>,
    fast_reload: bool,
}

#[derive(Clone)]
pub enum Handle {
    Static(Environment<'static>),
    Autoreload(Arc<AutoReloader>),
}

pub struct AutoReloader(minijinja_autoreload::AutoReloader);

#[derive(Debug, Error)]
#[error("failed to fetch/render template {0}")]
pub struct TemplateError(#[from] minijinja::Error);

#[derive(Debug, Error)]
#[error(transparent)]
pub struct EnvError(#[from] minijinja::Error);

#[derive(Debug, Error)]
#[error(transparent)]
pub struct GetTemplateError(#[from] minijinja::Error);

#[derive(Debug, Error)]
pub enum RenderTemplateError {
    #[error("failed to acquire environment. reason: {0}")]
    EnvError(#[from] EnvError),
    #[error("failed to get template from environment. reason: {0}")]
    GetTemplateError(#[from] GetTemplateError),
    #[error("f7ailed to render template with the provided context. reason: {0}")]
    RenderError(#[from] RenderError),
}

#[derive(Debug, Error)]
pub enum RenderTemplateStrError {
    #[error("failed to acquire environment. reason: {0}")]
    EnvError(#[from] EnvError),
    #[error("failed to render template string with the provided context. reason: {0}")]
    RenderError(#[from] RenderError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct RenderError(#[from] minijinja::Error);

////////////////////////////////////////////////////////////////////////////////

impl HandleBuilder {
    /// Returns a mutable borrow to the internal environment. This may be useful
    /// for adding stuff to the minijinja environment, e.g setting a path loader.
    pub fn get_mut_env(&mut self) -> &mut Environment<'static> {
        &mut self.env
    }

    pub fn set_watch_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.watch_paths = paths;
        self
    }

    pub fn set_fast_reload(mut self, fast_reload: bool) -> Self {
        self.fast_reload = fast_reload;
        self
    }

    pub fn build_autoreload(self) -> Handle {
        let autoreloader = minijinja_autoreload::AutoReloader::new(move |notifier| {
            for path in self.watch_paths.iter() {
                notifier.watch_path(path, true);
            }

            notifier.set_fast_reload(self.fast_reload);
            Ok(self.env.clone())
        });
        Handle::Autoreload(Arc::from(AutoReloader(autoreloader)))
    }

    pub fn build_static(self) -> Handle {
        Handle::Static(self.env)
    }
}

impl Handle {
    fn get_env(&self) -> Result<Environment<'static>, EnvError> {
        match self {
            Handle::Static(env) => Ok(env.clone()),
            Handle::Autoreload(ar) => Ok(ar.0.acquire_env()?.clone()),
        }
    }

    pub fn builder() -> HandleBuilder {
        HandleBuilder {
            env: Environment::new(),
            watch_paths: Vec::new(),
            fast_reload: false,
        }
    }

    pub fn render_template<S: Serialize>(
        &self,
        context: S,
        template_file: &str,
    ) -> Result<String, RenderTemplateError> {
        let env = self.get_env()?;

        let template = env
            .get_template(template_file)
            .map_err(|e| GetTemplateError(e))?;

        Ok(template.render(context).map_err(|e| RenderError(e))?)
    }

    pub fn render_template_str<S: Serialize>(
        &self,
        context: S,
        template: &str,
    ) -> Result<String, RenderTemplateStrError> {
        let env = self.get_env()?;
        let template = env.template_from_str(template).unwrap();
        Ok(template.render(context).map_err(|e| RenderError(e))?)
    }
}

impl std::fmt::Debug for AutoReloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AutoReloader")
    }
}
