#![forbid(unsafe_code)]
use std::{path::PathBuf, sync::Arc};

use minijinja::Environment;
use minijinja_autoreload::AutoReloader;
use serde::Serialize;

pub use minijinja::{context, Error};

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

#[derive(Debug)]
pub struct TemplateError(minijinja::Error);

impl std::error::Error for TemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<minijinja::Error> for TemplateError {
    fn from(value: minijinja::Error) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct EnvError(minijinja::Error);

impl std::error::Error for EnvError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<minijinja::Error> for EnvError {
    fn from(value: minijinja::Error) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct GetTemplateError(minijinja::Error);

impl std::error::Error for GetTemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<minijinja::Error> for GetTemplateError {
    fn from(value: minijinja::Error) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for GetTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum RenderTemplateError {
    EnvError(EnvError),
    GetTemplateError(GetTemplateError),
    RenderError(RenderError),
}

impl std::error::Error for RenderTemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for RenderTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderTemplateError::EnvError(env_error) => {
                write!(f, "failed to acquire environment {}", env_error)
            }
            RenderTemplateError::GetTemplateError(get_template_error) => {
                write!(f, "unable to get template from env {}", get_template_error)
            }
            RenderTemplateError::RenderError(render_error) => write!(
                f,
                "error in rendering with the provided context {}",
                render_error
            ),
        }
    }
}

impl From<EnvError> for RenderTemplateError {
    fn from(value: EnvError) -> Self {
        Self::EnvError(value)
    }
}

impl From<GetTemplateError> for RenderTemplateError {
    fn from(value: GetTemplateError) -> Self {
        Self::GetTemplateError(value)
    }
}

impl From<RenderError> for RenderTemplateError {
    fn from(value: RenderError) -> Self {
        Self::RenderError(value)
    }
}

#[derive(Debug)]
pub enum RenderTemplateStrError {
    EnvError(EnvError),
    RenderError(RenderError),
}

impl std::error::Error for RenderTemplateStrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<EnvError> for RenderTemplateStrError {
    fn from(value: EnvError) -> Self {
        Self::EnvError(value)
    }
}

impl From<RenderError> for RenderTemplateStrError {
    fn from(value: RenderError) -> Self {
        Self::RenderError(value)
    }
}

impl std::fmt::Display for RenderTemplateStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderTemplateStrError::EnvError(env_error) => {
                write!(f, "failed to acquire environment {}", env_error)
            }
            RenderTemplateStrError::RenderError(render_error) => write!(
                f,
                "error in rendering with the provided context {}",
                render_error
            ),
        }
    }
}

#[derive(Debug)]
pub struct RenderError(minijinja::Error);

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<minijinja::Error> for RenderError {
    fn from(value: minijinja::Error) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl HandleBuilder {
    /// Returns a mutable borrow to the internal environment. This may be useful
    /// for adding stuff to the minijinja environment, e.g setting a path loader.
    pub fn get_mut_env(&mut self) -> &mut Environment<'static> {
        &mut self.env
    }

    /// Watches the directories for any changes, and triggers the notifier.
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
        Handle::Autoreload(Arc::from(autoreloader))
    }

    pub fn build_static(self) -> Handle {
        Handle::Static(self.env)
    }
}

impl Handle {
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
        match self {
            Handle::Static(env) => {
                let template = env
                    .get_template(template_file)
                    .map_err(|e| GetTemplateError(e))?;

                Ok(template.render(context).map_err(|e| RenderError(e))?)
            }
            Handle::Autoreload(ar) => {
                let env = ar.acquire_env().unwrap();

                let template = env
                    .get_template(template_file)
                    .map_err(|e| GetTemplateError(e))?;

                Ok(template.render(context).map_err(|e| RenderError(e))?)
            }
        }
    }

    pub fn render_template_str<S: Serialize>(
        &self,
        context: S,
        template: &str,
    ) -> Result<String, RenderTemplateStrError> {
        match self {
            Handle::Static(env) => {
                let template = env.template_from_str(template).map_err(|e| EnvError(e))?;
                Ok(template.render(context).map_err(|e| RenderError(e))?)
            }
            Handle::Autoreload(ar) => {
                let env = ar.acquire_env().map_err(|e| EnvError(e))?;
                let template = env.template_from_str(template).unwrap();
                Ok(template.render(context).map_err(|e| RenderError(e))?)
            }
        }
    }
}
