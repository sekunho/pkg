#![forbid(unsafe_code)]

use std::sync::Arc;

use minijinja::Environment;

pub use minijinja::{context, Error};

pub struct HandleBuilder {
    env: Environment<'static>,
}

#[derive(Clone)]
pub struct Handle(Arc<Environment<'static>>);

#[derive(Debug)]
pub struct GetTemplateError(minijinja::Error);

impl std::error::Error for GetTemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Display for GetTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderTemplateError {
    GetTemplateError(GetTemplateError),
    RenderError(RenderError),
}

impl std::error::Error for RenderTemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderTemplateError::GetTemplateError(err) => Some(err),
            RenderTemplateError::RenderError(err) => Some(err),
        }
    }
}

impl std::fmt::Display for RenderTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderTemplateError::GetTemplateError(get_template_error) => {
                write!(f, "unable to get template from env {get_template_error}")
            }
            RenderTemplateError::RenderError(render_error) => write!(
                f,
                "error in rendering with the provided context {render_error}"
            ),
        }
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
pub struct CompileError(minijinja::Error);

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderTemplateStrError {
    CompileError(CompileError),
    RenderError(RenderError),
}

impl std::error::Error for RenderTemplateStrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderTemplateStrError::CompileError(err) => Some(err),
            RenderTemplateStrError::RenderError(err) => Some(err),
        }
    }
}

impl From<CompileError> for RenderTemplateStrError {
    fn from(value: CompileError) -> Self {
        Self::CompileError(value)
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
            RenderTemplateStrError::CompileError(err) => {
                write!(f, "failed to compile template {err}")
            }
            RenderTemplateStrError::RenderError(render_error) => write!(
                f,
                "error in rendering with the provided context {render_error}"
            ),
        }
    }
}

#[derive(Debug)]
pub struct RenderError(minijinja::Error);

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
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

    pub fn build(self) -> Handle {
        Handle(Arc::new(self.env))
    }
}

impl Handle {
    #[must_use]
    pub fn builder() -> HandleBuilder {
        HandleBuilder {
            env: Environment::new(),
        }
    }

    pub fn render_template<C: Into<minijinja::Value>>(
        &self,
        context: C,
        template_file: &str,
    ) -> Result<String, RenderTemplateError> {
        let template = self
            .0
            .get_template(template_file)
            .map_err(GetTemplateError)?;

        Ok(template.render(context).map_err(RenderError)?)
    }

    pub fn render_template_str<C: Into<minijinja::Value>>(
        &self,
        context: C,
        template: &str,
    ) -> Result<String, RenderTemplateStrError> {
        let template = self.0.template_from_str(template).map_err(CompileError)?;
        Ok(template.render(context).map_err(RenderError)?)
    }
}
