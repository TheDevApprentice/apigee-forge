use std::path::{Path, PathBuf};

use crate::{domain::render::RenderedBundle, error::BundleError};

pub trait BundleWriter: Send + Sync {
    fn write(&self, bundle: &RenderedBundle, output: &Path) -> Result<PathBuf, BundleError>;
}
