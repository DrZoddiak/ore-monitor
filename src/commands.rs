use crate::ore::OreClient;
use crate::sponge_schemas::{
    Category, PaginatedProjectResult, PaginatedVersionResult, Project, ProjectSortingStrategy,
    Version,
};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use common::{query, Query};
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use std::{
    fmt::Display,
    io::Cursor,
    path::{Path, PathBuf},
};

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

/// Represents the "root" commands
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

/// Enables the searching of plugins based on a query if provided
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

/// Retreives project information about a plugin
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

/// Provides common code for commands to use
struct CommonCommandHandle {}

impl CommonCommandHandle {
    async fn get_plugin_response(plugin_id: &String, ore_client: &OreClient) -> Result<Response> {
        let link = format!("/projects/{}", plugin_id);
        Ok(ore_client.get(link, None).await?)
    }
}

/// Represents subcommands of [PluginCommand]
#[derive(Subcommand)]
enum PluginSubCommand {
    /// Shows a list of available versions
    Versions(PluginVersionCommand),
}

/// A subcommand of [PluginCommand] that shows all available versions
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

/// A command to Install plugins
#[derive(Parser)]
pub struct InstallCommand {
    /// Directory to install into
    #[arg(short, long)]
    dir: Option<PathBuf>,
    /// The plugin id to install
    plugin_id: String,
    /// The version to install
    version: String,
}

impl InstallCommand {
    const DEFAULT_FILE_NAME: &'static str = "unknown_file";
    pub fn extract_filename(headers: &str) -> Option<&str> {
        let start = headers.find('"')?;
        let end = headers.rfind('"')?;
        (start != end).then_some(&headers[start + 1..end])
    }
}

#[async_trait]
impl OreCommand for InstallCommand {
    async fn handle(&self, ore_client: OreClient, _link_query: Option<Query>) -> Result<()> {
        // This whole command is basically a workaround for the API not having a download link available
        // This response allows me to generate the owner:slug information for a valid link to download
        let res = CommonCommandHandle::get_plugin_response(&self.plugin_id, &ore_client).await?;

        let proj: Project = self.serialize(res).await?;

        // This is a link for the main website, in the same way users would
        // retrieve a file.
        let link = format!(
            "/{}/{}/versions/{}/download",
            proj.namespace.owner, proj.namespace.slug, self.version
        );

        // get_install uses a modified base_url to function
        let res = ore_client.get_install(link, None).await?;

        // Proper error handling is needed here
        // should probably check for a successful status code instead
        if res.status() == StatusCode::NOT_FOUND {
            return Err(anyhow::Error::msg(
                "Resource not available, ensure you're using a valid ID & Version!",
            ));
        }

        // Because we don't install from the API, we have to retrieve the file name from where available.
        let file_name = res
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|s| Some(s.to_str()))
            .and_then(|f| Some(f.unwrap_or(Self::DEFAULT_FILE_NAME)))
            .and_then(|header| Self::extract_filename(header))
            .unwrap_or(Self::DEFAULT_FILE_NAME);

        let dir = self
            .dir
            .as_deref()
            .unwrap_or_else(|| Path::new(""))
            .display()
            .to_string();

        let dir = dir + file_name;

        let mut file = std::fs::File::create(&dir)?;
        let mut content = Cursor::new(res.bytes().await?);

        std::io::copy(&mut content, &mut file)?;

        Ok(println!("Installed {}", dir))
    }
}
