use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use skills_manage::{
    import_skill, load_or_init_config, pull_sources, recommend, register_tool, save_config,
    search_skills, sync_tools,
};

#[derive(Parser)]
#[command(
    name = "skills-manage",
    version,
    about = "统一管理本地/外部 skills，并同步到不同 AI 编程工具"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long)]
        global_store: Option<PathBuf>,
    },
    Import {
        path: PathBuf,
        #[arg(long)]
        id: Option<String>,
    },
    List,
    RegisterTool {
        tool: String,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    Sync,
    Pull,
    Search {
        keyword: String,
    },
    Recommend,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = load_or_init_config()?;

    match cli.command {
        Commands::Init { global_store } => {
            if let Some(store) = global_store {
                cfg.global_store = store;
                save_config(&cfg)?;
            }
            println!("initialized at {}", cfg.global_store.display());
        }
        Commands::Import { path, id } => {
            let record = import_skill(&cfg, &path, id)?;
            println!("imported {} v{}", record.id, record.version);
        }
        Commands::List => {
            let manifest = skills_manage::load_manifest(&cfg.global_store)?;
            for skill in manifest.values() {
                println!("{}\t{}\t{}", skill.id, skill.version, skill.description);
            }
            if manifest.is_empty() {
                println!("no skills");
            }
        }
        Commands::RegisterTool { tool, path } => {
            let target = register_tool(&mut cfg, &tool, path)?;
            save_config(&cfg)?;
            println!("registered {} => {}", target.tool, target.path.display());
        }
        Commands::Sync => {
            let logs = sync_tools(&cfg)?;
            for l in logs {
                println!("{l}");
            }
        }
        Commands::Pull => {
            let logs = pull_sources(&cfg)?;
            for l in logs {
                println!("{l}");
            }
        }
        Commands::Search { keyword } => {
            for item in search_skills(&cfg, &keyword)? {
                println!("{item}");
            }
        }
        Commands::Recommend => {
            for s in recommend(&cfg)? {
                println!("{}\tpopularity:{}\t{}", s.id, s.popularity, s.description);
            }
        }
    }

    Ok(())
}
