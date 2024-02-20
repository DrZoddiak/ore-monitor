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
                        if let Some(value) = e {
                            Some(vec![value.to_string()])
                        } else {
                            None
                        }
                    },
                    QueryType::Vec(e) => {
                        if let Some(value) = e {
                            Some(value.iter().map(|f| f.to_string()).collect())
                        } else {
                            None
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

#[derive(Subcommand)]
enum PluginVersions {
    Version {
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        #[arg(short, long)]
        limit: Option<i64>,
        #[arg(short, long)]
        offset: Option<i64>,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        stats: Option<bool>,
    },
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
    async fn handle(&self, ore_client: OreClient) -> Result<()> {
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
        Ok(ProjectHandle::new(ore_client, Some(e))
            .await
            .projects()
            .await?)
    }
}

#[derive(Parser)]
struct PluginCommand {
    plugin_id: String,
    #[command(flatten)]
    versions: Option<PluginVersionsCommand>,
}

#[derive(Parser)]
struct PluginVersionsCommand {
    name: Option<String>,
    stats: Option<bool>,
    tags: Option<Vec<String>>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl PluginCommand {
    async fn handle(&self, ore_client: OreClient) -> Result<()> {
        let query = Ok(if let Some(versions) = self.versions.as_ref() {
            query!(
                "q" : QueryType::Value(&Some(&self.plugin_id)),
                "name" : QueryType::Value(&versions.name),
                "stats" : QueryType::Value(&versions.stats),
                "tags" : QueryType::Vec(&versions.tags),
                "limit" : QueryType::Value(&versions.limit),
                "offset" : QueryType::Value(&versions.offset)
            )
        } else {
            query!(
                "q" : QueryType::Value(&Some(&self.plugin_id)),
            )
        })?;

        let mut proj_handle = ProjectHandle::new(ore_client, Some(query)).await;
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
                SubCommands::Search(cmd) => Ok(cmd.handle(ore_client).await?),

                SubCommands::Plugin(cmd) => Ok(cmd.handle(ore_client).await?),
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
    let parsed = Cli::parse();
    handle_cli(parsed).await.expect("No error")
}
