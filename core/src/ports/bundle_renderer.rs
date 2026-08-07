use async_trait::async_trait;

use crate::{
    domain::render::{RenderInput, RenderedBundle},
    domain::Template,
    error::RenderError,
};

#[async_trait]
pub trait BundleRenderer: Send + Sync {
    async fn render(
        &self,
        input: &RenderInput,
        template: &Template,
    ) -> Result<RenderedBundle, RenderError>;
}
