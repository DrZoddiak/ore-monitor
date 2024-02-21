use crate::ore;
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use ore::{OreClient, ProjectHandle};
use std::{collections::HashMap, fmt::Display};

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

enum QueryType<'a, T: Display> {
    Vec(&'a Option<Vec<T>>),
    Value(&'a Option<T>),
}

#[derive(Parser)]
#[command(version)]
pub enum Cli {
    Projects {
        #[command(subcommand)]
        search: Option<SubCommands>,
    },
}

#[derive(Subcommand)]
pub enum SubCommands {
    Search(SearchCommand),
    Plugin(PluginCommand),
}

#[derive(Parser)]
pub struct SearchCommand {
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
    pub async fn handle(&self, ore_client: &OreClient) -> Result<()> {
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
pub struct PluginCommand {
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
    pub async fn handle(&self, ore_client: &OreClient) -> Result<()> {
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
