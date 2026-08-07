use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use apigee_forge_core::{
    domain::{ProxyName, RenderInput, RenderMethod, RenderRoute, TargetUrl, Template},
    infra::{TeraBundleRenderer, ZipBundleArchiver},
    ports::{BundleArchiver, BundleRenderer},
};
use quick_xml::{events::Event, Reader};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
struct GoldenFixture {
    input: GoldenInput,
    expected_files: Vec<ExpectedFile>,
    forbidden_fragments: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GoldenInput {
    proxy_name: String,
    target_url: String,
    routes: Vec<GoldenRoute>,
}

#[derive(Debug, Deserialize)]
struct GoldenRoute {
    path: String,
    method: RenderMethod,
    security_requirements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedFile {
    path: String,
    root: String,
    contains: Vec<String>,
}

fn temporary_directory() -> Result<PathBuf, Box<dyn Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!(
        "apigee-forge-golden-bundle-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir(&path)?;
    Ok(path)
}

fn write_test_report(report: &Value) -> Result<PathBuf, Box<dyn Error>> {
    let report_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("test-results");
    fs::create_dir_all(&report_directory)?;
    let report_path = report_directory.join("generated_bundle_validation.json");
    let report_file = File::create(&report_path)?;
    serde_json::to_writer_pretty(report_file, report)?;
    Ok(report_path)
}

fn xml_root(contents: &str) -> Result<String, Box<dyn Error>> {
    let mut reader = Reader::from_str(contents);
    loop {
        match reader.read_event()? {
            Event::Start(event) | Event::Empty(event) => {
                return Ok(String::from_utf8(event.name().as_ref().to_vec())?);
            }
            Event::Eof => return Err("XML document has no root element".into()),
            _ => {}
        }
    }
}

fn render_reference_bundle(
    fixture: &GoldenFixture,
) -> Result<apigee_forge_core::domain::RenderedBundle, Box<dyn Error>> {
    let input = RenderInput::new(
        ProxyName::try_new(fixture.input.proxy_name.clone())?,
        TargetUrl::try_new(fixture.input.target_url.clone())?,
        fixture
            .input
            .routes
            .iter()
            .map(|route| RenderRoute {
                path: route.path.clone(),
                method: route.method,
                security_requirements: route.security_requirements.clone(),
            })
            .collect(),
    );
    let template: Template =
        serde_json::from_str(include_str!("../../schemas/template.example.json"))?;
    let renderer = TeraBundleRenderer::new()?;
    let runtime = Runtime::new()?;
    Ok(runtime.block_on(renderer.render(&input, &template))?)
}

#[test]
fn validates_reference_proxy_bundle_fixture() -> Result<(), Box<dyn Error>> {
    let fixture: GoldenFixture =
        serde_json::from_str(include_str!("fixtures/proxy_bundle.golden.json"))?;
    let bundle = render_reference_bundle(&fixture)?;
    let expected_paths = fixture
        .expected_files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let actual_paths = bundle
        .files
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_paths, expected_paths);

    let expected_by_path = fixture
        .expected_files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let mut roots = BTreeMap::new();
    for file in &bundle.files {
        let root = xml_root(&file.contents)?;
        let expected = expected_by_path
            .get(file.relative_path.as_str())
            .ok_or("generated file is not in golden fixture")?;
        assert_eq!(
            root, expected.root,
            "unexpected XML root for {}",
            file.relative_path
        );
        for fragment in &expected.contains {
            assert!(
                file.contents.contains(fragment),
                "missing expected fragment in {}: {fragment}",
                file.relative_path
            );
        }
        for forbidden in &fixture.forbidden_fragments {
            assert!(
                !file.contents.contains(forbidden),
                "forbidden fragment found in {}: {forbidden}",
                file.relative_path
            );
        }
        roots.insert(file.relative_path.clone(), root);
    }

    let root = temporary_directory()?;
    let archive_path = root.join("reference.zip");
    ZipBundleArchiver::new().archive(&bundle, &archive_path)?;
    let archive_file = File::open(&archive_path)?;
    let mut archive = ZipArchive::new(archive_file)?;
    let archive_paths = (0..archive.len())
        .map(|index| archive.by_index(index).map(|entry| entry.name().to_owned()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(archive_paths, expected_paths);
    assert!(archive_paths
        .iter()
        .all(|path| path.starts_with("apiproxy/")));

    let mut archived_roots = BTreeMap::new();
    for path in &archive_paths {
        let mut archived_file = archive.by_name(path)?;
        let mut contents = String::new();
        archived_file.read_to_string(&mut contents)?;
        archived_roots.insert(path.clone(), xml_root(&contents)?);
    }
    assert_eq!(archived_roots, roots);

    let report = json!({
        "test": "validates_reference_proxy_bundle_fixture",
        "golden_fixture": "core/tests/fixtures/proxy_bundle.golden.json",
        "generated_files": actual_paths,
        "xml_roots": roots,
        "archive_entries": archive_paths,
        "archive_root": "apiproxy/",
        "forbidden_fragments_found": false
    });
    let report_path = write_test_report(&report)?;
    eprintln!("test report: {}", report_path.display());

    fs::remove_dir_all(root)?;
    Ok(())
}
