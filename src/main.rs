mod ore;
mod paginated_project_result;

use anyhow::{Error, Result};
use clap::{arg, Parser, Subcommand};
use ore::{OreAuth, ProjectHandle};
use std::{collections::HashMap, fmt::Display};

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
enum SubCommands {
    Search {
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

async fn handle_cli(cli: Cli) -> Result<()> {
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
                    let e = query!(
                        "q" : QueryType::Value(search),
                        "categories" : QueryType::Vec(category),
                        "tags" : QueryType::Vec(tags),
                        "owner" : QueryType::Value(owner),
                        "sort" : QueryType::Value(sort),
                        "relevance" : QueryType::Value(relevance),
                        "limit" : QueryType::Value(limit),
                        "offset" : QueryType::Value(offset)
                    );

                    Ok(ProjectHandle::new(ore_client, Some(e))
                        .await
                        .projects()
                        .await?)
                }
                SubCommands::Plugin {
                    plugin_id,
                    versions,
                } => Ok({
                    let f = query!(
                        "q" : QueryType::Value(&Some(plugin_id))
                    );
                    match versions {
                        Some(PluginVersions::Version {
                            name,
                            stats,
                            tags,
                            limit,
                            offset,
                        }) => {
                            let e = query!(
                                "q" : QueryType::Value(&Some(plugin_id)),
                                "name" : QueryType::Value(name),
                                "stats" : QueryType::Value(stats),
                                "tags" : QueryType::Vec(tags),
                                "limit" : QueryType::Value(limit),
                                "offset" : QueryType::Value(offset)
                            );

                            let mut proj_handle = ProjectHandle::new(ore_client, Some(e)).await;
                            return Ok(proj_handle.plugin().await?);
                        }
                        None => {
                            let mut proj_handle = ProjectHandle::new(ore_client, Some(f)).await;

                            Ok::<(), Error>(proj_handle.plugin().await?)?
                        }
                    }
                }),
            },
            None => Ok(println!("Argument required!")),
        },
        None => return Ok(()),
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
