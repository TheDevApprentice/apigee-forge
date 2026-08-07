use std::{error::Error, fs, path::PathBuf, sync::Arc};

use apigee_forge_core::{
    domain::{ProxyName, RenderInput, RenderMethod, RenderRoute, TargetUrl, Template},
    infra::{FilesystemBundleWriter, TeraBundleRenderer, ZipBundleArchiver},
    openapi::{parse_openapi, HttpMethod},
    use_cases::GenerateProxyBundleUseCase,
};
use clap::{Args, Parser, Subcommand};
use serde_json::json;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(
    name = "apigee-forge",
    version,
    about = "Apigee Forge command line interface"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Format successful and error output as JSON"
    )]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum Commands {
    /// Authenticate the CLI for interactive or headless use
    Login(LoginArgs),
    /// Manage local proxy templates
    Template {
        #[command(subcommand)]
        command: TemplateCommand,
    },
    /// Generate a local Apigee proxy bundle
    Generate(GenerateArgs),
    /// Deploy a proxy bundle to Apigee
    Deploy(DeployArgs),
    /// Read a deployment status
    Status(StatusArgs),
    /// List proxies accessible in an Apigee organization
    ListProxies(ListProxiesArgs),
}

#[derive(Debug, Args, PartialEq, Eq)]
struct LoginArgs {
    #[arg(
        long,
        help = "Use GOOGLE_APPLICATION_CREDENTIALS without opening a browser"
    )]
    headless: bool,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum TemplateCommand {
    /// Import a template from a JSON file
    Create(TemplateCreateArgs),
    /// List locally stored templates
    List,
    /// Display a locally stored template
    Show(TemplateNameArgs),
    /// Replace a locally stored template from a JSON file
    Update(TemplateUpdateArgs),
    /// Delete a locally stored template
    Delete(TemplateNameArgs),
}

#[derive(Debug, Args, PartialEq, Eq)]
struct TemplateCreateArgs {
    #[arg(long, value_name = "FILE")]
    from: PathBuf,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct TemplateNameArgs {
    name: String,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct TemplateUpdateArgs {
    name: String,
    #[arg(long, value_name = "FILE")]
    from: PathBuf,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct GenerateArgs {
    #[arg(long, value_name = "FILE")]
    spec: PathBuf,
    #[arg(long, value_name = "FILE")]
    template: PathBuf,
    #[arg(long, value_name = "NAME")]
    proxy_name: String,
    #[arg(long, value_name = "DIRECTORY")]
    output: PathBuf,
    #[arg(long, value_name = "FILE")]
    archive: PathBuf,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct DeployArgs {
    #[arg(long)]
    org: String,
    #[arg(long)]
    environment: String,
    #[arg(long)]
    proxy_name: String,
    #[arg(long)]
    revision: u32,
    #[arg(long, value_name = "FILE")]
    bundle: PathBuf,
    #[arg(long)]
    override_existing: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct StatusArgs {
    deployment_id: String,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct ListProxiesArgs {
    #[arg(long)]
    org: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let json_output = cli.json;
    if let Err(error) = run(cli) {
        if json_output {
            eprintln!(
                "{}",
                json!({
                    "ok": false,
                    "command": "unknown",
                    "data": null,
                    "error": { "code": "COMMAND_FAILED", "message": error.to_string() }
                })
            );
        } else {
            eprintln!("error: {error}");
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Generate(arguments) => run_generate(arguments, cli.json),
        Commands::Login(_)
        | Commands::Template { .. }
        | Commands::Deploy(_)
        | Commands::Status(_)
        | Commands::ListProxies(_) => {
            Err(std::io::Error::other("command is reserved for a later M4 step").into())
        }
    }
}

fn run_generate(arguments: GenerateArgs, json_output: bool) -> Result<(), Box<dyn Error>> {
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

    if json_output {
        println!(
            "{}",
            json!({
                "ok": true,
                "command": "generate",
                "data": {
                    "proxy_name": result.proxy_name,
                    "rendered_file_count": result.rendered_file_count,
                    "bundle_directory": result.bundle_directory,
                    "archive_path": result.archive_path
                },
                "error": null
            })
        );
    } else {
        println!(
            "generated proxy={} files={} directory={} archive={}",
            result.proxy_name,
            result.rendered_file_count,
            result.bundle_directory.display(),
            result.archive_path.display()
        );
    }
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

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, GenerateArgs, TemplateCommand};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn parses_generate_command_and_global_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::try_parse_from([
            "apigee-forge",
            "--json",
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
        ])?;
        assert!(cli.json);
        assert_eq!(
            cli.command,
            Commands::Generate(GenerateArgs {
                spec: PathBuf::from("openapi.yaml"),
                template: PathBuf::from("template.json"),
                proxy_name: "orders-v1".to_owned(),
                output: PathBuf::from("out"),
                archive: PathBuf::from("out/orders.zip"),
            })
        );
        Ok(())
    }

    #[test]
    fn parses_the_complete_m4_command_tree_without_executing_it(
    ) -> Result<(), Box<dyn std::error::Error>> {
        for arguments in [
            vec!["apigee-forge", "login", "--headless"],
            vec!["apigee-forge", "template", "list"],
            vec!["apigee-forge", "template", "show", "standard"],
            vec![
                "apigee-forge",
                "deploy",
                "--org",
                "acme",
                "--environment",
                "prod",
                "--proxy-name",
                "orders",
                "--revision",
                "1",
                "--bundle",
                "orders.zip",
            ],
            vec!["apigee-forge", "status", "deployment-1"],
            vec!["apigee-forge", "list-proxies", "--org", "acme"],
        ] {
            assert!(Cli::try_parse_from(arguments).is_ok());
        }
        let template = Cli::try_parse_from(["apigee-forge", "template", "list"])?;
        assert!(matches!(
            template.command,
            Commands::Template {
                command: TemplateCommand::List
            }
        ));
        Ok(())
    }

    #[test]
    fn rejects_missing_required_generate_arguments_and_unknown_commands() {
        assert!(Cli::try_parse_from(["apigee-forge", "generate"]).is_err());
        assert!(Cli::try_parse_from(["apigee-forge", "login", "--unknown"]).is_err());
        assert!(Cli::try_parse_from(["apigee-forge", "future-command"]).is_err());
    }
}
