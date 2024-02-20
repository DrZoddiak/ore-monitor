mod ore;
mod paginated_project_result;

use anyhow::{Error, Result};
use clap::{arg, Parser, Subcommand};
use ore::{OreAuth, OreClient, ProjectHandle};
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
    Projects {
        #[command(subcommand)]
        search: Option<SubCommands>,
    },
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

struct SearchCommand {
    search: Option<String>,
    category: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    owner: Option<String>,
    sort: Option<String>,
    relevance: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl SearchCommand {
    fn new(
        search: Option<String>,
        category: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        owner: Option<String>,
        sort: Option<String>,
        relevance: Option<bool>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Self {
        SearchCommand {
            search,
            category,
            tags,
            owner,
            sort,
            relevance,
            limit,
            offset,
        }
    }

    async fn handle(self, ore_client: OreClient) -> Result<()> {
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

struct PluginCommand {
    plugin_id: String,
    name: Option<String>,
    stats: Option<bool>,
    tags: Option<Vec<String>>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl PluginCommand {
    fn new(
        plugin_id: String,
        name: Option<String>,
        stats: Option<bool>,
        tags: Option<Vec<String>>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Self {
        PluginCommand {
            plugin_id,
            name,
            stats,
            tags,
            limit,
            offset,
        }
    }

    async fn handle(self, ore_client: OreClient) -> Result<()> {
        let e = query!(
            "q" : QueryType::Value(&Some(self.plugin_id)),
            "name" : QueryType::Value(&self.name),
            "stats" : QueryType::Value(&self.stats),
            "tags" : QueryType::Vec(&self.tags),
            "limit" : QueryType::Value(&self.limit),
            "offset" : QueryType::Value(&self.offset)
        );

        let mut proj_handle = ProjectHandle::new(ore_client, Some(e)).await;
        return Ok(proj_handle.plugin().await?);
    }
}

async fn handle_cli(cli: Cli) -> Result<()> {
    //Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    //parse command
    match &cli.command {
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
                    SearchCommand::new(
                        search.clone(),
                        category.clone(),
                        tags.clone(),
                        owner.clone(),
                        sort.clone(),
                        *relevance,
                        *limit,
                        *offset,
                    )
                    .handle(ore_client)
                    .await
                }

                SubCommands::Plugin {
                    plugin_id,
                    versions,
                } => match versions {
                    Some(PluginVersions::Version {
                        name,
                        stats,
                        tags,
                        limit,
                        offset,
                    }) => {
                        PluginCommand::new(
                            plugin_id.to_string(),
                            name.clone(),
                            *stats,
                            tags.clone().clone(),
                            *limit,
                            *offset,
                        )
                        .handle(ore_client)
                        .await
                    }

                    None => {
                        let mut proj_handle = ProjectHandle::new(
                            ore_client,
                            Some(query!(
                                "q" : QueryType::Value(&Some(plugin_id))
                            )),
                        )
                        .await;

                        Ok(Ok::<(), Error>(proj_handle.plugin().await?)?)
                    }
                },
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
