mod ore;
mod paginated_project_result;

use anyhow::{Ok, Result};
use clap::{arg, Parser, Subcommand};
use ore::{OreAuth, OreClient, ProjectHandle};
use std::{collections::HashMap, fmt::Display};

#[derive(Parser)]
#[command(version)]
enum Cli {
    Projects {
        #[command(subcommand)]
        search: Option<SubCommands>,
    },
}

#[derive(Subcommand)]
enum SubCommands {
    Search(SearchCommand),
    Plugin(PluginCommand),
}

macro_rules! query {
    ($($lit:literal : $val:expr),+ $(,)?) => {
      {
        let mut map: HashMap<String, Vec<String>> = Default::default();

            $(
                let arg = match $val {
                    QueryType::Value(e) => {
                        match e {
                            Some(value) => Some(vec![value.to_string()]),
                            _ => None
                        }
                    },
                    QueryType::Vec(e) => {
                        match e {
                            Some(value) => Some(value.iter().map(|f| f.to_string()).collect()),
                            _ => None
                        }
                    },
                };

                if let Some(args) = arg {
                    map.insert($lit.to_string(), args)
                } else {
                    None
                }
            ;)+

            let mut vec: Vec<(String, String)> = vec![];
            map.iter().for_each(|f| {
                f.1.iter().for_each(|e| vec.push((f.0.to_string(), e.to_string())))
            });
            vec
      }
    }
}

#[derive(Parser)]
struct SearchCommand {
    search: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    category: Option<Vec<String>>,
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
    #[arg(short, long)]
    owner: Option<String>,
    #[arg(short, long)]
    sort: Option<String>,
    #[arg(short, long)]
    relevance: Option<bool>,
    #[arg(short, long)]
    limit: Option<i64>,
    #[arg(long)]
    offset: Option<i64>,
}

impl SearchCommand {
    async fn handle(&self, ore_client: &OreClient) -> Result<()> {
        let e = query!(
            "q" : QueryType::Value(&self.search),
            "categories" : QueryType::Vec(&self.category),
            "tags" : QueryType::Vec(&self.tags),
            "owner" : QueryType::Value(&self.owner),
            "sort" : QueryType::Value(&self.sort),
            "relevance" : QueryType::Value(&self.relevance),
            "limit" : QueryType::Value(&self.limit),
            "offset" : QueryType::Value(&self.offset)
        );
        Ok(ProjectHandle::new(ore_client, Some(e)).projects().await?)
    }
}

#[derive(Parser)]
struct PluginCommand {
    plugin_id: String,
    #[command(subcommand)]
    versions: Option<PluginSubCommand>,
}

#[derive(Subcommand)]
enum PluginSubCommand {
    Version(PluginVersion),
}

#[derive(Parser)]
struct PluginVersion {
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
    #[arg(short, long)]
    limit: Option<i64>,
    #[arg(long)]
    offset: Option<i64>,
}

impl PluginSubCommand {
    async fn handle(&self, plugin_id: &String, ore_client: &OreClient) -> Result<()> {
        let query = match self {
            Self::Version(cmd) => {
                query!(
                    "q" : QueryType::Value(&Some(&plugin_id)),
                    "tags" : QueryType::Vec(&cmd.tags),
                    "limit" : QueryType::Value(&cmd.limit),
                    "offset" : QueryType::Value(&cmd.offset)
                )
            }
        };

        let mut proj_handle = ProjectHandle::new(ore_client, Some(query));
        return Ok(proj_handle.plugin_version().await?);
    }
}

impl PluginCommand {
    async fn handle(&self, ore_client: &OreClient) -> Result<()> {
        let query = query!(
            "q" : QueryType::Value(&Some(&self.plugin_id)),
        );

        if let Some(ver) = &self.versions {
            return Ok(ver.handle(&self.plugin_id, ore_client).await?);
        }

        let mut proj_handle = ProjectHandle::new(ore_client, Some(query));

        return Ok(proj_handle.plugin().await?);
    }
}

async fn handle_cli(cli: Cli) -> Result<()> {
    //Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    //parse command
    match &cli {
        Cli::Projects { search } => match search {
            Some(subcmd) => match subcmd {
                SubCommands::Search(cmd) => Ok(cmd.handle(&ore_client).await?),

                SubCommands::Plugin(cmd) => Ok(cmd.handle(&ore_client).await?),
            },
            None => Ok(println!("Argument required!")),
        },
    }
}

enum QueryType<'a, T: Display> {
    Vec(&'a Option<Vec<T>>),
    Value(&'a Option<T>),
}

#[tokio::main]
async fn main() {
    handle_cli(Cli::parse()).await.expect("No error")
}
