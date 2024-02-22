use crate::{
    models::{PaginatedProjectResult, PaginatedVersionResult, Project},
    ore,
};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use ore::OreClient;
use reqwest::Response;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, fmt::Display};

macro_rules! query {
    ($($lit:literal : $val:expr),+ $(,)?) => {
        {
            let mut map: HashMap<String, Vec<String>> = Default::default();

            $(
                if let Some(args) = $val.into() {
                    map.insert($lit.to_string(), args)
                } else {
                    None
                };
            )+


            map.iter().map( |k| {
                k.1.iter().map(|v| (k.0.to_string(), v.to_string()))
            }).flatten().collect::<Vec<(String,String)>>()
        }
    }
}

enum QueryType<'a, T: Display> {
    Vec(&'a Option<Vec<T>>),
    Value(&'a Option<T>),
}

impl<'a, T: Display> Into<Option<Vec<String>>> for QueryType<'a, T> {
    fn into(self) -> Option<Vec<String>> {
        match self {
            QueryType::Value(Some(e)) => Some(vec![e.to_string().to_lowercase()]),
            QueryType::Vec(Some(e)) => Some(e.iter().map(|f| f.to_string()).collect()),
            _ => None,
        }
    }
}

#[derive(Parser)]
#[command(version)]
pub enum Cli {
    /// Main entrypoint for the commands.
    Projects {
        /// A [`None`] value will return a list of plugins
        #[command(subcommand)]
        subcommand: SubCommands,
    },
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Allows for searching for a list of plugins based off of the query
    Search(SearchCommand),
    /// Retreives a plugin from its plugin_id
    Plugin(PluginCommand),
}

#[derive(Parser)]
pub struct SearchCommand {
    /// A search query
    search: Option<String>,
    /// A comma separated list of Categories
    #[arg(short, long, value_delimiter = ',')]
    category: Option<Vec<String>>,
    /// A comma seperated list of Tags
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
    /// Searches for plugins from an Owner
    #[arg(short, long)]
    owner: Option<String>,
    /// How to sort the plugins
    #[arg(short, long)]
    sort: Option<String>,
    /// Should relevance be considered when sorting projects
    #[arg(short, long)]
    relevance: Option<bool>,
    /// The maximum amount of plugins to display
    #[arg(short, long)]
    limit: Option<i64>,
    /// Where to begin displaying the list from
    #[arg(long)]
    offset: Option<i64>,
}

/// Represents a regular Command
#[async_trait]
pub trait OreCommand {
    async fn handle(&self, ore_client: &OreClient) -> Result<()>;
}

#[async_trait]
impl OreCommand for SearchCommand {
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
pub struct PluginCommand {
    /// The plugin ID to search by
    plugin_id: String,
    /// A Subcommand for displaying versions of the plugin
    #[command(subcommand)]
    versions: Option<PluginSubCommand>,
}

#[derive(Subcommand)]
enum PluginSubCommand {
    /// The version Subcommand
    Versions(PluginVersion),
}

#[derive(Parser)]
struct PluginVersion {
    /// Comma separated list of Tags
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
    /// The limit of versions to display
    #[arg(short, long)]
    limit: Option<i64>,
    /// Where to begin display the list from
    #[arg(long)]
    offset: Option<i64>,
}

impl PluginSubCommand {
    async fn handle(&self, plugin_id: &String, ore_client: &OreClient) -> Result<()> {
        let query = match self {
            Self::Versions(cmd) => {
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

#[async_trait]
impl OreCommand for PluginCommand {
    async fn handle(&self, ore_client: &OreClient) -> Result<()> {
        if let Some(ver) = &self.versions {
            return Ok(ver.handle(&self.plugin_id, ore_client).await?);
        }

        let query = query!(
            "q" : QueryType::Value(&Some(&self.plugin_id)),
        );

        let mut proj_handle = ProjectHandle::new(ore_client, Some(query));

        return Ok(proj_handle.plugin().await?);
    }
}

pub struct ProjectHandle<'a> {
    ore_client: &'a OreClient,
    query: Option<Vec<(String, String)>>,
}

impl<'a> ProjectHandle<'a> {
    pub fn new(ore_client: &'a OreClient, query: Option<Vec<(String, String)>>) -> Self {
        ProjectHandle { ore_client, query }
    }

    // Gets projects from query input
    pub(crate) async fn projects(&mut self) -> Result<()> {
        let res: Response = if let Some(query) = &self.query {
            self.ore_client
                .get_url_query("/projects".to_string(), query.to_vec())
                .await?
        } else {
            return Ok(());
        };
        let res: PaginatedProjectResult = Self::serialize(Self::handle_response(res).await?)?;
        Ok(Self::display_results(res))
    }

    pub(crate) async fn plugin(&mut self) -> Result<()> {
        let res: Response = if let Some(query) = &self.query {
            let link = format!("/projects/{}", query.first().unwrap().1);
            self.ore_client.get_url(link).await?
        } else {
            return Ok(());
        };
        let res: Project = Self::serialize(Self::handle_response(res).await?)?;
        Ok(print!("{}", res))
    }

    pub async fn plugin_version(&mut self) -> Result<()> {
        let res: Response = if let Some(query) = &self.query {
            let link = format!(
                "/projects/{}/versions",
                query
                    .iter()
                    .filter(|k| k.0 == "q")
                    .map(|f| f.1.clone())
                    .collect::<String>()
            );
            self.ore_client.get_url_query(link, query.to_vec()).await?
        } else {
            return Ok(());
        };
        let res: PaginatedVersionResult = Self::serialize(Self::handle_response(res).await?)?;
        Ok(print!("{}", res))
    }

    // Displays the results for Projects
    fn display_results(result: PaginatedProjectResult) {
        result
            .result
            .iter()
            .for_each(|proj| println!("{}", proj.plugin_id))
    }

    fn serialize<T: DeserializeOwned>(txt: String) -> Result<T> {
        serde_json::from_str(&txt).map_err(|e| anyhow::Error::from(e))
    }

    // Common method for projects to handle responses.
    async fn handle_response(res: Response) -> Result<String> {
        Ok(res.text().await?)
    }
}
