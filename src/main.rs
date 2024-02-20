mod errors;
mod ore;
mod paginated_project_result;

use std::{collections::HashMap, fmt::Display};

use clap::{Parser, Subcommand};
use errors::OreError;
use ore::{OreAuth, ProjectHandle};
use serde::Serialize;

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// optional subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Permissions {},
    Projects {
        #[command(subcommand)]
        search: Option<SubCommands>,
    },
}

#[derive(Subcommand)]
enum SubCommands {
    Search {
        search: Option<String>,
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long)]
        tags: Option<String>,
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
    },
    Plugin {
        plugin_id: String,
        #[command(subcommand)]
        versions: Option<PluginVersions>,
    },
}

#[derive(Subcommand)]
enum PluginVersions {
    Version {
        #[arg(short, long)]
        tags: Option<String>,
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

async fn handle_cli(cli: Cli) -> Result<(), OreError> {
    //Authorize the ore client
    let mut ore_client = OreAuth::new().auth().await?;

    //parse command
    match &cli.command {
        Some(Commands::Permissions {}) => match ore_client.permissions().await {
            Ok(res) => Ok(res),
            Err(_) => return Ok(()),
        },

        Some(Commands::Projects { search }) => match search {
            Some(subcmd) => match subcmd {
                SubCommands::Search {
                    search,
                    category,
                    tags,
                    owner,
                    sort,
                    relevance,
                    limit,
                    offset,
                } => {
                    let bld = QueryBuilder::new()
                        .add_query("q".to_string(), search)
                        .add_vec("categories".to_string(), category)
                        .add_vec("tags".to_string(), tags)
                        .add_query("owner".to_string(), owner)
                        .add_query("sort".to_string(), sort)
                        .add_query("relevance".to_string(), relevance)
                        .add_query("limit".to_string(), limit)
                        .add_query("offset".to_string(), offset)
                        .build();

                    Ok(ProjectHandle::new(ore_client, Some(bld))
                        .await
                        .projects()
                        .await?)
                }
                SubCommands::Plugin {
                    plugin_id,
                    versions,
                } => {
                    let bld = QueryBuilder::new().add_query("q".to_string(), &Some(plugin_id));

                    if let Some(versions) = versions {
                        match versions {
                            PluginVersions::Version {
                                name,
                                stats,
                                tags,
                                limit,
                                offset,
                            } => {
                                let bld = bld
                                    .add_query("name".to_string(), name)
                                    .add_query("stats".to_string(), stats)
                                    .add_vec("tags".to_string(), tags)
                                    .add_query("limit".to_string(), limit)
                                    .add_query("offset".to_string(), offset);

                                if name.is_some() {}
                                if stats.is_some() {}

                                let mut proj_handle =
                                    ProjectHandle::new(ore_client, Some(bld.build())).await;
                                return Ok(proj_handle.plugin().await?);
                            }
                        }
                    }
                    let mut proj_handle = ProjectHandle::new(ore_client, Some(bld.build())).await;

                    Ok(proj_handle.plugin().await?)
                }
            },

            None => Ok(println!("Argument required!")),
        },

        None => return Ok(()),
    }
}

fn parse_list(value: String) -> Option<Vec<String>> {
    Some(value.split(',').map(|f| f.to_string()).collect())
}

#[derive(Serialize, Debug)]
struct QueryBuilder {
    query: HashMap<String, Vec<String>>,
}

impl QueryBuilder {
    fn new() -> Self {
        QueryBuilder {
            query: HashMap::new(),
        }
    }

    fn add_query<T: Display>(mut self, key: String, value: &Option<T>) -> Self {
        if let Some(value) = value {
            self.query.insert(key, vec![value.to_string()]);
        }
        self
    }

    fn add_vec(mut self, key: String, value: &Option<String>) -> Self {
        if let Some(value) = value {
            let str: Vec<String> = parse_list(value.to_string()).unwrap();
            self.query.insert(key, str);
        };

        self
    }

    fn build(self) -> Vec<(String, String)> {
        let mut vec: Vec<(String, String)> = vec![];
        self.query.iter().for_each(|f| {
            f.1.iter()
                .for_each(|e| vec.push((f.0.to_string(), e.to_string())))
        });
        vec
    }
}

#[tokio::main]
async fn main() {
    let parsed = Cli::parse();
    handle_cli(parsed).await.expect("No error")
}
