use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{Seek, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use zip::{write::SimpleFileOptions, CompressionMethod, DateTime, ZipWriter};

use crate::{
    domain::render::{RenderedBundle, RenderedFile},
    error::BundleError,
    ports::BundleArchiver,
};

static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);
const ZIP_COMPRESSION_LEVEL: i64 = 6;

pub struct ZipBundleArchiver;

impl ZipBundleArchiver {
    pub fn new() -> Self {
        Self
    }

    fn validate_bundle(bundle: &RenderedBundle) -> Result<Vec<&RenderedFile>, BundleError> {
        if bundle.files.is_empty() {
            return Err(BundleError::EmptyBundle);
        }

        let mut paths = HashSet::with_capacity(bundle.files.len());
        let mut files = Vec::with_capacity(bundle.files.len());
        for file in &bundle.files {
            validate_archive_path(file)?;
            if !paths.insert(file.relative_path.as_str()) {
                return Err(BundleError::InvalidFilePath);
            }
            files.push(file);
        }

        let has_proxy_endpoint = paths.contains("apiproxy/proxies/default.xml");
        let has_target_endpoint = paths.contains("apiproxy/targets/default.xml");
        if !has_proxy_endpoint || !has_target_endpoint {
            return Err(BundleError::IncompleteBundle);
        }

        files.sort_unstable_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Ok(files)
    }

    fn output_staging_path(output: &Path) -> Result<PathBuf, BundleError> {
        let parent = output.parent().unwrap_or_else(|| Path::new("."));
        let file_name = output
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(BundleError::InvalidOutputPath)?;
        let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(parent.join(format!(
            ".{file_name}.staging-{}-{counter}",
            std::process::id()
        )))
    }
}

impl Default for ZipBundleArchiver {
    fn default() -> Self {
        Self::new()
    }
}

impl BundleArchiver for ZipBundleArchiver {
    fn archive(&self, bundle: &RenderedBundle, output: &Path) -> Result<PathBuf, BundleError> {
        let files = Self::validate_bundle(bundle)?;
        validate_output_path(output)?;
        let staging = Self::output_staging_path(output)?;
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging)
            .map_err(|_| BundleError::Io)?;

        let result = write_archive(file, &files).and_then(|file| {
            file.sync_all().map_err(|_| BundleError::Io)?;
            fs::rename(&staging, output).map_err(|_| BundleError::Io)
        });
        if result.is_err() {
            let _ = fs::remove_file(&staging);
        }
        result.map(|_| output.to_path_buf())
    }
}

fn write_archive<W: Write + Seek>(writer: W, files: &[&RenderedFile]) -> Result<W, BundleError> {
    let timestamp =
        DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).map_err(|_| BundleError::Zip)?;
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(ZIP_COMPRESSION_LEVEL))
        .last_modified_time(timestamp)
        .unix_permissions(0o644);
    let mut archive = ZipWriter::new(writer);
    for file in files {
        archive
            .start_file(&file.relative_path, options)
            .map_err(|_| BundleError::Zip)?;
        archive
            .write_all(file.contents.as_bytes())
            .map_err(|_| BundleError::Io)?;
    }
    archive.finish().map_err(|_| BundleError::Zip)
}

fn validate_output_path(output: &Path) -> Result<(), BundleError> {
    if output.as_os_str().is_empty() {
        return Err(BundleError::InvalidOutputPath);
    }
    match fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.file_type().is_dir() => {
            Err(BundleError::InvalidOutputPath)
        }
        Ok(_) => Err(BundleError::OutputAlreadyExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = output.parent().unwrap_or_else(|| Path::new("."));
            match fs::symlink_metadata(parent) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                    Err(BundleError::InvalidOutputPath)
                }
                Ok(_) => Ok(()),
                Err(_) => Err(BundleError::Io),
            }
        }
        Err(_) => Err(BundleError::Io),
    }
}

fn validate_archive_path(file: &RenderedFile) -> Result<(), BundleError> {
    let relative_path = &file.relative_path;
    let path = Path::new(relative_path);
    if relative_path.is_empty()
        || path.is_absolute()
        || relative_path.contains('\\')
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(BundleError::InvalidFilePath);
    }

    let components = path.components().collect::<Vec<_>>();
    if components.len() != 3 || component_name(components[0])? != "apiproxy" {
        return Err(BundleError::InvalidFilePath);
    }
    let directory = component_name(components[1])?;
    if !matches!(directory, "proxies" | "targets" | "policies" | "resources") {
        return Err(BundleError::InvalidFilePath);
    }
    let file_name = component_name(components[2])?;
    if !is_safe_file_name(file_name) {
        return Err(BundleError::InvalidFilePath);
    }
    if directory != "resources" && !file_name.ends_with(".xml") {
        return Err(BundleError::InvalidFilePath);
    }
    Ok(())
}

fn component_name(component: Component<'_>) -> Result<&str, BundleError> {
    match component {
        Component::Normal(value) => value.to_str().ok_or(BundleError::InvalidFilePath),
        _ => Err(BundleError::InvalidFilePath),
    }
}

fn is_safe_file_name(file_name: &str) -> bool {
    !file_name.is_empty()
        && file_name != "."
        && file_name != ".."
        && file_name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        io::{Cursor, Read, Seek, SeekFrom, Write},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use zip::ZipArchive;

    use crate::{
        domain::render::{RenderedBundle, RenderedFile},
        error::BundleError,
        infra::ZipBundleArchiver,
        ports::BundleArchiver,
    };

    fn temporary_directory(name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("apigee-forge-{name}-{timestamp}"));
        fs::create_dir(&path)?;
        Ok(path)
    }

    fn valid_bundle() -> Result<RenderedBundle, Box<dyn Error>> {
        Ok(RenderedBundle::new(vec![
            RenderedFile::try_new("apiproxy/targets/default.xml", "<TargetEndpoint />")?,
            RenderedFile::try_new("apiproxy/proxies/default.xml", "<ProxyEndpoint />")?,
            RenderedFile::try_new("apiproxy/policies/Quota.xml", "<Quota />")?,
        ]))
    }

    #[test]
    fn writes_exact_sorted_zip_entries_and_extracts_control_files() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("zip-success")?;
        let output = root.join("proxy.zip");
        ZipBundleArchiver::new().archive(&valid_bundle()?, &output)?;

        let file = File::open(&output)?;
        let mut archive = ZipArchive::new(file)?;
        let names = (0..archive.len())
            .map(|index| archive.by_index(index).map(|entry| entry.name().to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            names,
            vec![
                "apiproxy/policies/Quota.xml",
                "apiproxy/proxies/default.xml",
                "apiproxy/targets/default.xml"
            ]
        );

        let mut proxy = archive.by_name("apiproxy/proxies/default.xml")?;
        let mut proxy_contents = String::new();
        proxy.read_to_string(&mut proxy_contents)?;
        assert_eq!(proxy_contents, "<ProxyEndpoint />");
        assert!(names.iter().all(|name| name.starts_with("apiproxy/")));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn creates_deterministic_archives_independent_of_input_order() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("zip-deterministic")?;
        let first = root.join("first.zip");
        let second = root.join("second.zip");
        let bundle = valid_bundle()?;
        let reversed = RenderedBundle::new(bundle.files.iter().rev().cloned().collect());
        ZipBundleArchiver::new().archive(&bundle, &first)?;
        ZipBundleArchiver::new().archive(&reversed, &second)?;
        assert_eq!(fs::read(first)?, fs::read(second)?);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_empty_incomplete_and_unsafe_bundles() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("zip-invalid")?;
        let writer = ZipBundleArchiver::new();
        assert!(matches!(
            writer.archive(&RenderedBundle::new(Vec::new()), &root.join("empty.zip")),
            Err(BundleError::EmptyBundle)
        ));
        let incomplete = RenderedBundle::new(vec![RenderedFile {
            relative_path: "apiproxy/policies/Quota.xml".to_owned(),
            contents: "<Quota />".to_owned(),
        }]);
        assert!(matches!(
            writer.archive(&incomplete, &root.join("incomplete.zip")),
            Err(BundleError::IncompleteBundle)
        ));
        let unsafe_bundle = RenderedBundle::new(vec![
            RenderedFile {
                relative_path: "apiproxy/proxies/default.xml".to_owned(),
                contents: "<ProxyEndpoint />".to_owned(),
            },
            RenderedFile {
                relative_path: "apiproxy/targets/../evil.xml".to_owned(),
                contents: "<TargetEndpoint />".to_owned(),
            },
        ]);
        assert!(matches!(
            writer.archive(&unsafe_bundle, &root.join("unsafe.zip")),
            Err(BundleError::InvalidFilePath)
        ));
        assert!(!root.join("unsafe.zip").exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn reports_archive_finalization_error() -> Result<(), Box<dyn Error>> {
        let archive = zip::ZipWriter::new(FailingWriter::new());
        let result = archive.finish().map_err(|_| BundleError::Zip);
        assert!(matches!(result, Err(BundleError::Zip)));
        Ok(())
    }

    struct FailingWriter {
        cursor: Cursor<Vec<u8>>,
        writes: usize,
    }

    impl FailingWriter {
        fn new() -> Self {
            Self {
                cursor: Cursor::new(Vec::new()),
                writes: 0,
            }
        }
    }

    impl Write for FailingWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.writes += 1;
            if self.writes > 0 {
                return Err(std::io::Error::other("intentional finalization failure"));
            }
            self.cursor.write(bytes)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.cursor.flush()
        }
    }

    impl Seek for FailingWriter {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            self.cursor.seek(position)
        }
    }
}
