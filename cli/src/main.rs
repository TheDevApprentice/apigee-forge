use std::{env, error::Error, fs, path::PathBuf, sync::Arc};

mod auth;
mod output;

use apigee_forge_core::{
    domain::{ProxyName, RenderInput, RenderMethod, RenderRoute, TargetUrl, Template},
    infra::{
        FilesystemBundleWriter, FilesystemTemplateRepository, ReqwestApigeeGateway,
        TeraBundleRenderer, ZipBundleArchiver,
    },
    openapi::{parse_openapi, HttpMethod},
    ports::{ApigeeDeploymentGateway, ApigeeGateway, TemplateRepository},
    use_cases::{
        CreateTemplateUseCase, DeleteTemplateUseCase, DeployProxyUseCase,
        GenerateProxyBundleUseCase, GetDeploymentStatusUseCase, GetTemplateUseCase,
        ImportProxyBundleUseCase, ListProxiesUseCase, ListTemplatesUseCase, UpdateTemplateUseCase,
    },
};
use auth::{authenticate, build_auth_provider, select_auth_mode};
use clap::{Args, Parser, Subcommand};
use output::{classify_error, failure_json, human_message, success_json, ExitCode};
use serde_json::json;
use url::Url;

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
    Template(TemplateArgs),
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
        conflicts_with = "interactive",
        help = "Use GOOGLE_APPLICATION_CREDENTIALS without opening a browser"
    )]
    headless: bool,
    #[arg(
        long,
        conflicts_with = "headless",
        help = "Explicitly start the desktop OAuth browser flow"
    )]
    interactive: bool,
    #[arg(long, help = "Select the Apigee organization explicitly")]
    org: Option<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct TemplateArgs {
    #[arg(
        long,
        default_value = ".apigee-forge/templates",
        value_name = "DIRECTORY"
    )]
    directory: PathBuf,
    #[command(subcommand)]
    command: TemplateCommand,
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
    #[arg(long, value_name = "FILE", conflicts_with = "template_name")]
    template: Option<PathBuf>,
    #[arg(long, value_name = "NAME", conflicts_with = "template")]
    template_name: Option<String>,
    #[arg(
        long,
        value_name = "DIRECTORY",
        default_value = ".apigee-forge/templates",
        requires = "template_name"
    )]
    template_dir: PathBuf,
    #[arg(long, value_name = "NAME")]
    proxy_name: String,
    #[arg(long, value_name = "DIRECTORY")]
    output: PathBuf,
    #[arg(long, value_name = "FILE")]
    archive: PathBuf,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct DeployArgs {
    #[arg(long, conflicts_with = "interactive")]
    headless: bool,
    #[arg(long, conflicts_with = "headless")]
    interactive: bool,
    #[arg(long)]
    org: Option<String>,
    #[arg(long)]
    environment: String,
    #[arg(long)]
    proxy_name: String,
    #[arg(long, value_name = "FILE")]
    bundle: PathBuf,
    #[arg(long)]
    override_existing: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct StatusArgs {
    #[arg(long, conflicts_with = "interactive")]
    headless: bool,
    #[arg(long, conflicts_with = "headless")]
    interactive: bool,
    #[arg(long)]
    org: Option<String>,
    #[arg(long)]
    environment: String,
    #[arg(long)]
    proxy_name: String,
    #[arg(long)]
    revision: u32,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct ListProxiesArgs {
    #[arg(long, conflicts_with = "interactive")]
    headless: bool,
    #[arg(long, conflicts_with = "headless")]
    interactive: bool,
    #[arg(long)]
    org: Option<String>,
}

fn main() {
    let arguments = env::args_os().collect::<Vec<_>>();
    let json_output = arguments.iter().any(|argument| argument == "--json");
    let cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(_) => {
            if json_output {
                let failure = output::SafeFailure {
                    code: "INVALID_ARGUMENTS",
                    exit_code: ExitCode::Usage,
                    message: "command line arguments are invalid",
                };
                if let Ok(document) = failure_json("parse", &failure) {
                    println!("{document}");
                }
            } else {
                eprintln!("error: invalid command line arguments");
            }
            std::process::exit(ExitCode::Usage.as_i32());
        }
    };
    let json_output = cli.json;
    let command = command_name(&cli.command);
    if let Err(error) = run(cli) {
        let failure = classify_error(error.as_ref());
        if json_output {
            if let Ok(document) = failure_json(command, &failure) {
                println!("{document}");
            }
        } else {
            eprintln!("error: {}", human_message(&failure));
        }
        std::process::exit(failure.exit_code.as_i32());
    }
    std::process::exit(ExitCode::Success.as_i32());
}

fn command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Login(_) => "login",
        Commands::Template(_) => "template",
        Commands::Generate(_) => "generate",
        Commands::Deploy(_) => "deploy",
        Commands::Status(_) => "status",
        Commands::ListProxies(_) => "list-proxies",
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Generate(arguments) => run_generate(arguments, cli.json),
        Commands::Template(arguments) => run_template(arguments, cli.json),
        Commands::Login(arguments) => run_login(arguments, cli.json),
        Commands::ListProxies(arguments) => run_list_proxies(arguments, cli.json),
        Commands::Deploy(arguments) => run_deploy(arguments, cli.json),
        Commands::Status(arguments) => run_status(arguments, cli.json),
    }
}

fn run_login(arguments: LoginArgs, json_output: bool) -> Result<(), Box<dyn Error>> {
    let selection = select_auth_mode(arguments.headless, arguments.interactive)?;
    let provider = build_auth_provider(selection)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let context = runtime.block_on(authenticate(provider))?;
    let organization = auth::resolve_organization(&context, arguments.org.as_deref())?;
    let mut summary = auth::summary(&context);
    summary.selected_organization = Some(organization);
    if json_output {
        println!("{}", success_json("login", summary)?);
    } else {
        println!(
            "authenticated mode={} identity={} project_id={} organization={}",
            summary.mode,
            summary.identity.as_deref().unwrap_or("none"),
            summary.project_id.as_deref().unwrap_or("none"),
            summary.selected_organization.as_deref().unwrap_or("none")
        );
    }
    Ok(())
}

fn run_deploy(arguments: DeployArgs, json_output: bool) -> Result<(), Box<dyn Error>> {
    let selection = select_auth_mode(arguments.headless, arguments.interactive)?;
    let provider = build_auth_provider(selection)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let context = runtime.block_on(authenticate(provider.clone()))?;
    let organization = auth::resolve_organization(&context, arguments.org.as_deref())?;
    let bundle = fs::read(&arguments.bundle)?;
    let gateway_url = Url::parse("https://apigee.googleapis.com/v1/")?;
    let gateway = Arc::new(ReqwestApigeeGateway::new(gateway_url, provider)?);
    let imported = runtime.block_on(ImportProxyBundleUseCase::new(gateway.clone()).execute(
        &organization,
        &arguments.proxy_name,
        bundle,
    ))?;
    let deployment = runtime.block_on(DeployProxyUseCase::new(gateway).execute(
        &organization,
        &arguments.environment,
        &arguments.proxy_name,
        imported.number,
        arguments.override_existing,
    ))?;
    if json_output {
        println!("{}", success_json("deploy", deployment)?);
    } else {
        println!(
            "deployed proxy={} environment={} revision={} status={:?}",
            deployment.proxy_name, deployment.environment, deployment.revision, deployment.status
        );
    }
    Ok(())
}

fn run_status(arguments: StatusArgs, json_output: bool) -> Result<(), Box<dyn Error>> {
    let selection = select_auth_mode(arguments.headless, arguments.interactive)?;
    let provider = build_auth_provider(selection)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let context = runtime.block_on(authenticate(provider.clone()))?;
    let organization = auth::resolve_organization(&context, arguments.org.as_deref())?;
    let gateway_url = Url::parse("https://apigee.googleapis.com/v1/")?;
    let gateway: Arc<dyn ApigeeDeploymentGateway> =
        Arc::new(ReqwestApigeeGateway::new(gateway_url, provider)?);
    let deployment = runtime.block_on(GetDeploymentStatusUseCase::new(gateway).execute(
        &organization,
        &arguments.environment,
        &arguments.proxy_name,
        arguments.revision,
    ))?;
    if json_output {
        println!("{}", success_json("status", deployment)?);
    } else {
        println!(
            "status proxy={} environment={} revision={} state={:?}",
            deployment.proxy_name, deployment.environment, deployment.revision, deployment.status
        );
    }
    Ok(())
}

fn run_list_proxies(arguments: ListProxiesArgs, json_output: bool) -> Result<(), Box<dyn Error>> {
    let selection = select_auth_mode(arguments.headless, arguments.interactive)?;
    let provider = build_auth_provider(selection)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let context = runtime.block_on(authenticate(provider.clone()))?;
    let organization = auth::resolve_organization(&context, arguments.org.as_deref())?;
    let gateway_url = Url::parse("https://apigee.googleapis.com/v1/")?;
    let gateway: Arc<dyn ApigeeGateway> =
        Arc::new(ReqwestApigeeGateway::new(gateway_url, provider)?);
    let proxies = runtime.block_on(ListProxiesUseCase::new(gateway).execute(&organization))?;
    if json_output {
        println!("{}", success_json("list-proxies", proxies)?);
    } else {
        for proxy in proxies {
            println!("{}", proxy.name);
        }
    }
    Ok(())
}

fn run_template(arguments: TemplateArgs, json_output: bool) -> Result<(), Box<dyn Error>> {
    let repository: Arc<dyn TemplateRepository> =
        Arc::new(FilesystemTemplateRepository::new(arguments.directory));
    match arguments.command {
        TemplateCommand::Create(arguments) => {
            let template = load_template(&arguments.from)?;
            let name = template.metadata.name.clone();
            CreateTemplateUseCase::new(repository).execute(template)?;
            print_template_result(json_output, "create", json!({ "name": name }))?;
        }
        TemplateCommand::List => {
            let templates = ListTemplatesUseCase::new(repository).execute()?;
            print_template_result(json_output, "list", templates)?;
        }
        TemplateCommand::Show(arguments) => {
            let template = GetTemplateUseCase::new(repository).execute(&arguments.name)?;
            print_template_result(json_output, "show", template)?;
        }
        TemplateCommand::Update(arguments) => {
            let template = load_template(&arguments.from)?;
            if template.metadata.name != arguments.name {
                return Err(std::io::Error::other("template name does not match --name").into());
            }
            UpdateTemplateUseCase::new(repository).execute(template)?;
            print_template_result(json_output, "update", json!({ "name": arguments.name }))?;
        }
        TemplateCommand::Delete(arguments) => {
            DeleteTemplateUseCase::new(repository).execute(&arguments.name)?;
            print_template_result(json_output, "delete", json!({ "name": arguments.name }))?;
        }
    }
    Ok(())
}

fn load_template(path: &std::path::Path) -> Result<Template, Box<dyn Error>> {
    Ok(Template::from_json_str(&fs::read_to_string(path)?)?)
}

fn print_template_result<T: serde::Serialize>(
    json_output: bool,
    action: &str,
    data: T,
) -> Result<(), Box<dyn Error>> {
    if json_output {
        println!("{}", success_json(&format!("template {action}"), data)?);
    } else if matches!(action, "list" | "show") {
        println!("{}", serde_json::to_string_pretty(&data)?);
    } else {
        println!("template {action} succeeded");
    }
    Ok(())
}

fn load_generate_template(arguments: &GenerateArgs) -> Result<Template, Box<dyn Error>> {
    match (&arguments.template, &arguments.template_name) {
        (Some(path), None) => load_template(path),
        (None, Some(name)) => {
            let repository = FilesystemTemplateRepository::new(&arguments.template_dir);
            Ok(GetTemplateUseCase::new(Arc::new(repository)).execute(name)?)
        }
        (Some(_), Some(_)) => {
            Err(std::io::Error::other("--template and --template-name cannot be combined").into())
        }
        (None, None) => {
            Err(std::io::Error::other("one of --template or --template-name is required").into())
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
    let template = load_generate_template(&arguments)?;
    let input = RenderInput::new(
        ProxyName::try_new(arguments.proxy_name)?,
        target_url,
        routes,
    );

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
        let document = success_json(
            "generate",
            json!({
                "proxy_name": result.proxy_name,
                "rendered_file_count": result.rendered_file_count,
                "bundle_directory": result.bundle_directory,
                "archive_path": result.archive_path
            }),
        )?;
        println!("{document}");
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
    use super::{Cli, Commands, GenerateArgs, TemplateArgs, TemplateCommand};
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
                template: Some(PathBuf::from("template.json")),
                template_name: None,
                template_dir: PathBuf::from(".apigee-forge/templates"),
                proxy_name: "orders-v1".to_owned(),
                output: PathBuf::from("out"),
                archive: PathBuf::from("out/orders.zip"),
            })
        );
        Ok(())
    }

    #[test]
    fn parses_generate_with_repository_template_name() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::try_parse_from([
            "apigee-forge",
            "generate",
            "--spec",
            "openapi.yaml",
            "--template-name",
            "standard",
            "--template-dir",
            "templates",
            "--proxy-name",
            "orders-v1",
            "--output",
            "out",
            "--archive",
            "out/orders.zip",
        ])?;
        assert!(matches!(cli.command, Commands::Generate(GenerateArgs {
            template: None,
            template_name: Some(name),
            ..
        }) if name == "standard"));
        assert!(Cli::try_parse_from([
            "apigee-forge",
            "generate",
            "--spec",
            "openapi.yaml",
            "--template",
            "template.json",
            "--template-name",
            "standard",
            "--proxy-name",
            "orders-v1",
            "--output",
            "out",
            "--archive",
            "out/orders.zip",
        ])
        .is_err());
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
                "--headless",
                "--org",
                "acme",
                "--environment",
                "prod",
                "--proxy-name",
                "orders",
                "--bundle",
                "orders.zip",
            ],
            vec![
                "apigee-forge",
                "status",
                "--headless",
                "--org",
                "acme",
                "--environment",
                "prod",
                "--proxy-name",
                "orders",
                "--revision",
                "1",
            ],
            vec![
                "apigee-forge",
                "list-proxies",
                "--headless",
                "--org",
                "acme",
            ],
        ] {
            assert!(Cli::try_parse_from(arguments).is_ok());
        }
        let template = Cli::try_parse_from(["apigee-forge", "template", "list"])?;
        assert!(matches!(
            template.command,
            Commands::Template(TemplateArgs {
                command: TemplateCommand::List,
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn rejects_missing_required_generate_arguments_unknown_commands_and_duplicates() {
        assert!(Cli::try_parse_from(["apigee-forge", "generate"]).is_err());
        assert!(Cli::try_parse_from(["apigee-forge", "login", "--unknown"]).is_err());
        assert!(Cli::try_parse_from(["apigee-forge", "future-command"]).is_err());
        assert!(Cli::try_parse_from([
            "apigee-forge",
            "generate",
            "--spec",
            "one.yaml",
            "--spec",
            "two.yaml",
            "--template",
            "template.json",
            "--proxy-name",
            "orders-v1",
            "--output",
            "out",
            "--archive",
            "out/orders.zip"
        ])
        .is_err());
    }
}
