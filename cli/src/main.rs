use std::{env, error::Error, fs, path::PathBuf, sync::Arc};

use apigee_forge_core::{
    domain::{ProxyName, RenderInput, RenderMethod, RenderRoute, TargetUrl, Template},
    infra::{FilesystemBundleWriter, TeraBundleRenderer, ZipBundleArchiver},
    openapi::{parse_openapi, HttpMethod},
    use_cases::GenerateProxyBundleUseCase,
};

const USAGE: &str = "Usage: cli generate --spec <openapi.yaml> --template <template.json> --proxy-name <name> --output <directory> --archive <bundle.zip>";

#[derive(Debug, PartialEq, Eq)]
struct GenerateArgs {
    spec: PathBuf,
    template: PathBuf,
    proxy_name: String,
    output: PathBuf,
    archive: PathBuf,
}

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let arguments = parse_generate_args(&arguments).map_err(std::io::Error::other)?;
    let openapi_source = fs::read_to_string(&arguments.spec)?;
    let parsed_openapi = parse_openapi(&openapi_source)?;
    let target_url = TargetUrl::try_new(parsed_openapi.primary_server()?.to_owned())?;
    let routes = parsed_openapi
        .routes
        .into_iter()
        .map(|route| RenderRoute {
            path: route.path,
            method: render_method(route.method),
            security_requirements: route.security_requirements,
        })
        .collect();
    let input = RenderInput::new(
        ProxyName::try_new(arguments.proxy_name)?,
        target_url,
        routes,
    );
    let template: Template = serde_json::from_str(&fs::read_to_string(&arguments.template)?)?;

    let renderer = Arc::new(TeraBundleRenderer::new()?);
    let writer = Arc::new(FilesystemBundleWriter::new());
    let archiver = Arc::new(ZipBundleArchiver::new());
    let use_case = GenerateProxyBundleUseCase::new(renderer, writer, archiver);
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(use_case.execute(
        &input,
        &template,
        &arguments.output,
        &arguments.archive,
    ))?;

    println!(
        "generated proxy={} files={} directory={} archive={}",
        result.proxy_name,
        result.rendered_file_count,
        result.bundle_directory.display(),
        result.archive_path.display()
    );
    Ok(())
}

fn render_method(method: HttpMethod) -> RenderMethod {
    match method {
        HttpMethod::Get => RenderMethod::Get,
        HttpMethod::Put => RenderMethod::Put,
        HttpMethod::Post => RenderMethod::Post,
        HttpMethod::Delete => RenderMethod::Delete,
        HttpMethod::Options => RenderMethod::Options,
        HttpMethod::Head => RenderMethod::Head,
        HttpMethod::Patch => RenderMethod::Patch,
        HttpMethod::Trace => RenderMethod::Trace,
    }
}

fn parse_generate_args(arguments: &[String]) -> Result<GenerateArgs, String> {
    if arguments.first().map(String::as_str) != Some("generate") {
        return Err(USAGE.to_owned());
    }

    let mut spec = None;
    let mut template = None;
    let mut proxy_name = None;
    let mut output = None;
    let mut archive = None;
    let mut index = 1;
    while index < arguments.len() {
        let flag = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}\n{USAGE}"))?
            .clone();
        match flag {
            "--spec" => set_once(&mut spec, PathBuf::from(value), flag)?,
            "--template" => set_once(&mut template, PathBuf::from(value), flag)?,
            "--proxy-name" => set_once(&mut proxy_name, value, flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), flag)?,
            "--archive" => set_once(&mut archive, PathBuf::from(value), flag)?,
            _ => return Err(format!("unknown argument: {flag}\n{USAGE}")),
        }
        index += 2;
    }

    Ok(GenerateArgs {
        spec: spec.ok_or_else(|| format!("missing --spec\n{USAGE}"))?,
        template: template.ok_or_else(|| format!("missing --template\n{USAGE}"))?,
        proxy_name: proxy_name.ok_or_else(|| format!("missing --proxy-name\n{USAGE}"))?,
        output: output.ok_or_else(|| format!("missing --output\n{USAGE}"))?,
        archive: archive.ok_or_else(|| format!("missing --archive\n{USAGE}"))?,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        Err(format!("duplicate argument: {flag}\n{USAGE}"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_generate_args, GenerateArgs};
    use std::path::PathBuf;

    #[test]
    fn parses_only_the_minimal_generate_command() -> Result<(), Box<dyn std::error::Error>> {
        let arguments = [
            "generate",
            "--spec",
            "openapi.yaml",
            "--template",
            "template.json",
            "--proxy-name",
            "orders-v1",
            "--output",
            "out",
            "--archive",
            "out/orders.zip",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        assert_eq!(
            parse_generate_args(&arguments)?,
            GenerateArgs {
                spec: PathBuf::from("openapi.yaml"),
                template: PathBuf::from("template.json"),
                proxy_name: "orders-v1".to_owned(),
                output: PathBuf::from("out"),
                archive: PathBuf::from("out/orders.zip"),
            }
        );
        Ok(())
    }

    #[test]
    fn rejects_commands_outside_m3_scope() {
        let arguments = vec!["login".to_owned()];
        assert!(parse_generate_args(&arguments).is_err());
    }
}
