use std::path::{Path, PathBuf};

use crate::{domain::render::RenderedBundle, error::BundleError};

pub trait BundleArchiver: Send + Sync {
    fn archive(&self, bundle: &RenderedBundle, output: &Path) -> Result<PathBuf, BundleError>;
}
