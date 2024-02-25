use crate::{
    ore,
    sponge_schemas::{
        Category, PaginatedProjectResult, PaginatedVersionResult, Project, ProjectSortingStrategy,
        Version,
    },
};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use ore::OreClient;
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, fmt::Display, io::Cursor};

/// Builds a set of arguments to build a query for a link
/// Returns a [Vec]<([String],[String])>
///
/// Takes a [str] and [QueryType]
/// ```
/// query! {
///     // Would return [("q","value")]
///     "q" : QueryType::Value(Some("value")),
///     // Would return [("list","one")("list","two")("list",three)]
///     "list" : QueryType::Vec(Some(vec!["one","two","three"])),
///     ...
/// }
/// ```
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


            let query = map.iter().map( |k| {
                k.1.iter().map(|v| (k.0.to_string(), v.to_string()))
            }).flatten().collect::<Vec<(String,String)>>();
            Query::new(query)
        }
    }
}

/// Differentiates the difference between a Vec and Non-Vec value
/// For the purposes of providing a clean [Display] impl
enum QueryType<T: Display> {
    Vec(Option<Vec<T>>),
    Value(Option<T>),
}

impl<T: Display> Into<Option<Vec<String>>> for QueryType<T> {
    fn into(self) -> Option<Vec<String>> {
        match self {
            QueryType::Value(Some(e)) => Some(vec![e.to_string().to_lowercase()]),
            QueryType::Vec(Some(e)) => Some(e.iter().map(|f| f.to_string()).collect()),
            _ => None,
        }
    }
}

pub struct Query {
    query: Vec<(String, String)>,
}

impl Query {
    fn new(query: Vec<(String, String)>) -> Self {
        Query { query }
    }

    pub fn get_query(&self, key: &str) -> String {
        self.query
            .iter()
            .filter(|k| k.0 == key)
            .map(|f| f.1.to_string())
            .collect::<String>()
    }

    fn to_vec(&self) -> Vec<(String, String)> {
        self.query.to_vec()
    }
}

/// Represents a regular Command
#[async_trait]
pub trait OreCommand {
    async fn handle(&self, ore_client: OreClient, link_query: Option<Query>) -> Result<()>;

    async fn serialize<T: DeserializeOwned>(&self, txt: Response) -> Result<T>
    where
        Self: Sized,
    {
        serde_json::from_str(&txt.text().await?).map_err(|e| anyhow::Error::from(e))
    }

    fn print_res<T: Display>(&self, res: T) -> Result<()>
    where
        Self: Sized,
    {
        Ok(println!("{}", res))
    }
}

#[derive(Parser)]
#[command(version)]
pub enum Cli {
    /// Allows for searching for a list of plugins based off of the query
    Search(SearchCommand),
    /// Retreives info about a plugin from its plugin_id
    Plugin(PluginCommand),
    /// Installs a plugin from a plugin_id
    Install(InstallCommand),
}

#[derive(Parser)]
pub struct SearchCommand {
    /// A search query
    search: Option<String>,
    /// A comma separated list of Categories
    #[arg(short, long, value_delimiter = ',')]
    category: Option<Vec<Category>>,
    /// A comma seperated list of Tags
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
    /// Searches for plugins from an Owner
    #[arg(short, long)]
    owner: Option<String>,
    /// How to sort the plugins
    #[arg(short, long)]
    sort: Option<ProjectSortingStrategy>,
    /// Should relevance be considered when sorting projects
    #[arg(short, long)]
    relevance: Option<bool>,
    /// The maximum amount of plugins to display
    #[arg(short, long)]
    limit: Option<i64>,
    /// Where to begin displaying the list from
    #[arg(long)]
    #[clap(default_value_t = 0)]
    offset: u64,
}

#[async_trait]
impl OreCommand for SearchCommand {
    async fn handle(&self, ore_client: OreClient, _link_query: Option<Query>) -> Result<()> {
        let query = query!(
            "q" : QueryType::Value(self.search.as_ref()),
            "categories" : QueryType::Vec(self.category.clone()),
            "tags" : QueryType::Vec(self.tags.clone()),
            "owner" : QueryType::Value(self.owner.as_ref()),
            "sort" : QueryType::Value(self.sort.as_ref()),
            "relevance" : QueryType::Value(self.relevance),
            "limit" : QueryType::Value(self.limit),
            "offset" : QueryType::Value(Some(self.offset))
        );

        let res = ore_client
            .get("/projects".to_string(), Some(query.to_vec()))
            .await?;

        let res: PaginatedProjectResult = self.serialize(res).await?;

        Ok(self.print_res(res)?)
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

#[async_trait]
impl OreCommand for PluginCommand {
    async fn handle(&self, ore_client: OreClient, _link_query: Option<Query>) -> Result<()> {
        let query = query!(
            "plugin_id" : QueryType::Value(Some(&self.plugin_id)),
        );

        if let Some(ver) = &self.versions {
            return Ok(ver.handle(ore_client, Some(query)).await?);
        }

        let res =
            CommonCommandHandle::get_plugin_response(&query.get_query("plugin_id"), &ore_client)
                .await?;

        let res: Project = self.serialize(res).await?;

        Ok(self.print_res(res)?)
    }
}

struct CommonCommandHandle {}

impl CommonCommandHandle {
    async fn get_plugin_response(plugin_id: &String, ore_client: &OreClient) -> Result<Response> {
        let link = format!("/projects/{}", plugin_id);
        Ok(ore_client.get(link, None).await?)
    }
}

#[derive(Subcommand)]
enum PluginSubCommand {
    /// The version Subcommand
    Versions(PluginVersionCommand),
}

#[derive(Parser)]
struct PluginVersionCommand {
    /// Version ID to inspect
    name: Option<String>,
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

#[async_trait]
impl OreCommand for PluginSubCommand {
    async fn handle(&self, ore_client: OreClient, link_query: Option<Query>) -> Result<()> {
        let Self::Versions(cmd) = self;

        let query = query!(
            "tags" : QueryType::Vec(cmd.tags.clone()),
            "limit" : QueryType::Value(cmd.limit),
            "offset" : QueryType::Value(cmd.offset)
        )
        .to_vec();

        let link = format!(
            "/projects/{}/versions",
            link_query.unwrap().get_query("plugin_id")
        );

        if let Some(name) = &cmd.name {
            let link = format!("{}/{}", link, name);
            let res = ore_client.get(link, Some(query)).await?;
            let res: Version = self.serialize(res).await?;
            return Ok(self.print_res(res)?);
        }

        let res = ore_client.get(link, Some(query)).await?;
        let res: PaginatedVersionResult = self.serialize(res).await?;

        return Ok(self.print_res(res)?);
    }
}

#[derive(Parser)]
pub struct InstallCommand {
    plugin_id: String,
    version: String,
}

#[async_trait]
impl OreCommand for InstallCommand {
    async fn handle(&self, ore_client: OreClient, _link_query: Option<Query>) -> Result<()> {
        let res = CommonCommandHandle::get_plugin_response(&self.plugin_id, &ore_client).await?;

        let proj: Project = self.serialize(res).await?;

        let link = format!(
            "/{}/{}/versions/{}/download",
            proj.namespace.owner, proj.namespace.slug, self.version
        );

        let res = ore_client.get_install(link, None).await?;

        if res.status() == StatusCode::NOT_FOUND {
            return Err(anyhow::Error::msg(
                "Resource not available, ensure you're using a valid ID & Version!",
            ));
        }

        let default_file_name = "unknown_file_name";

        let file_name = if let Some(headers) = res.headers().get("content-disposition") {
            let (_, file_name) = headers.to_str()?.split_once('\"').unwrap_or_default();
            let (file_name, _) = file_name
                .rsplit_once('\"')
                .unwrap_or((default_file_name, ""));
            file_name
        } else {
            default_file_name
        };

        let dir = "local/".to_string() + file_name;

        let mut file = std::fs::File::create(&dir)?;
        let mut content = Cursor::new(res.bytes().await?);

        std::io::copy(&mut content, &mut file)?;

        Ok(println!("Installed {}", dir))
    }
}
