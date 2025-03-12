#[cfg(not(target_os = "linux"))]
compile_error!("can only be compiled on linux ;)");

use clap::{
    builder::{
        styling::{AnsiColor, Effects},
        Styles,
    },
    CommandFactory, Parser,
};
use clap_complete::{generate, Shell};
use gvm::{
    cli::{alias, init, install, list, list_remote, remove, remove_alias, update, use_version},
    Res,
};

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::White.on_default() | Effects::BOLD)
        .usage(AnsiColor::White.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightBlue.on_default())
        .placeholder(AnsiColor::BrightGreen.on_default())
}

#[derive(Parser, Debug, Clone)]
#[clap(
  version = env!("CARGO_PKG_VERSION"),
  name=env!("CARGO_PKG_NAME"),
  bin_name=env!("CARGO_PKG_NAME"),
  author=env!("CARGO_PKG_AUTHORS"),
  about=env!("CARGO_PKG_DESCRIPTION"),
  styles=styles(),
)]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug, Clone)]
enum Command {
    #[clap(about = "Install golang version from source")]
    Install(InstallOption),

    #[clap(about = "Remove installed verison of golang")]
    Remove(RemoveOption),

    #[clap(about = "Create alias for installed version")]
    Alias(AliasOption),

    #[clap(about = "Remove alias")]
    RemoveAlias(RemoveAliasOption),

    #[clap(about = "Show all installed versions", alias = "ls")]
    List(ListOption),

    #[clap(about = "List all available versions from source", alias = "ls-remote")]
    ListRemote(ListRemoteOption),

    #[clap(about = "Get shell completions")]
    Completions(CompletionsOption),

    #[clap(about = "Switch to specified version")]
    Use(UseOption),

    #[clap(about = "Update GVM version")]
    Update(UpdateOption),

    #[clap(about = "Init go environment")]
    Init(InitOption),
}

#[derive(Parser, Debug, Clone)]
struct InstallOption {
    #[clap(value_parser, index = 1)]
    version: String,

    #[clap(long, alias = "use")]
    use_version: bool,
}

#[derive(Parser, Debug, Clone)]
struct RemoveOption {
    #[clap(value_parser, index = 1)]
    version: String,
}

#[derive(Parser, Debug, Clone)]
struct AliasOption {
    #[clap(value_parser, index = 1)]
    alias: String,

    #[clap(value_parser, index = 2)]
    target: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct RemoveAliasOption {
    #[clap(value_parser, index = 1)]
    alias: String,
}

#[derive(Parser, Debug, Clone)]
struct ListOption {
    #[clap(value_parser, index = 1)]
    version: Option<String>,

    #[clap(long)]
    stable: bool,
}

#[derive(Parser, Debug, Clone)]
struct ListRemoteOption {
    #[clap(value_parser, index = 1)]
    version: Option<String>,

    #[clap(long)]
    stable: bool,
}

#[derive(Parser, Debug, Clone)]
struct UseOption {
    #[clap(value_parser, index = 1)]
    version: String,
}

#[derive(Parser, Debug, Clone)]
struct UpdateOption {}

#[derive(Parser, Debug, Clone)]
struct CompletionsOption {
    shell: Shell,
}

#[derive(Parser, Debug, Clone)]
struct InitOption {
    #[clap(value_parser, index = 1)]
    version: Option<String>,
}

#[tokio::main]
async fn main() -> Res<()> {
    let opts = Opts::parse();

    Ok(match opts.command {
        Command::Update(_opt) => {
            update().await?;
        }
        Command::Install(opt) => {
            install(opt.version, opt.use_version).await?;
        }
        Command::Remove(opt) => {
            remove(opt.version).await?;
        }
        Command::List(opt) => {
            list(opt.version, opt.stable).await?;
        }
        Command::ListRemote(opt) => {
            list_remote(opt.version, opt.stable).await?;
        }
        Command::Alias(opt) => {
            alias(opt.alias, opt.target).await?;
        }
        Command::RemoveAlias(opt) => {
            remove_alias(opt.alias).await?;
        }
        Command::Use(opt) => {
            use_version(opt.version).await?;
        }
        Command::Completions(opt) => {
            let mut cmd = Opts::command_for_update();
            let name = cmd.get_name().to_string();
            generate(opt.shell, &mut cmd, name, &mut std::io::stdout())
        }
        Command::Init(_opt) => {
            init().await?;
        }
    })
}
