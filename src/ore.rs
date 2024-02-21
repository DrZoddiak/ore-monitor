use std::{env, fmt::Display};

use anyhow::Ok;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{self, AUTHORIZATION},
    Client, RequestBuilder, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::paginated_project_result::PaginatedVersionResult;
use crate::paginated_project_result::{PaginatedProjectResult, Project};

#[derive(Debug)]
pub struct OreClient {
    client: Client,
    session: OreSession,
    base_url: String,
}

#[derive(Default, Debug)]
pub struct OreSession {
    session_id: String,
    expires: String,
}

impl OreSession {
    pub fn update(&mut self, response: OreAuthResponse) {
        self.session_id = response.session;
        self.expires = response.expires;
    }
}

#[derive(Debug)]
pub struct OreAuth {
    client: reqwest::Client,
    ore_session: OreSession,
    base_url: String,
    api_key: String,
}

impl Default for OreAuth {
    fn default() -> Self {
        OreAuth {
            client: Default::default(),
            ore_session: Default::default(),
            base_url: "https://ore.spongepowered.org/api/v2".to_string(),
            api_key: env::var("ORE_API_KEY")
                .unwrap_or("d08a6c8b-3a9e-44c1-9c85-a7dfedba00f5".to_string()),
        }
    }
}

/// Handles auth for Ore
impl OreAuth {
    /// Main method for authorizing, This is also how the OreClient is created
    pub(crate) async fn auth(mut self) -> Result<OreClient> {
        let res = self.send_request().await;
        let res = res?.text().await?;
        let res: OreAuthResponse = serde_json::from_str(&res)?;
        self.ore_session.update(res);

        Ok(OreClient::new(self.client, self.ore_session, self.base_url).await)
    }

    /// Send request for authentication
    async fn send_request(&self) -> Result<Response> {
        Ok(self
            .client
            .post(format!("{}/authenticate", self.base_url))
            .header(
                reqwest::header::WWW_AUTHENTICATE,
                format!("OreApi apikey={}", self.api_key),
            )
            .send()
            .await?)
    }
}

impl OreClient {
    pub async fn new(client: Client, session: OreSession, base_url: String) -> Self {
        OreClient {
            client,
            session,
            base_url,
        }
    }

    fn log(code: StatusCode) {
        let _msg = match code {
            StatusCode::NO_CONTENT => "Session Invalidated",
            StatusCode::BAD_REQUEST => "Request not made with a session",
            StatusCode::UNAUTHORIZED => "Api session missing, invalid, or expired",
            StatusCode::FORBIDDEN => "Not enough permission for endpoint",
            _ => "Status code undocumented",
        };
        //println!("Status Code :msg)
    }

    // Invalidates the current session
    pub(crate) async fn invalidate(&self) -> Result<()> {
        let builder = self
            .client
            .delete(format!("{}/sessions/current", self.base_url));
        let res = self.apply_headers(builder).send().await?;
        Self::log(res.status());
        Ok(())
    }

    // Applies auth headers
    fn apply_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        builder
            .header(
                reqwest::header::WWW_AUTHENTICATE,
                format!("OreApi session={}", self.session.session_id),
            )
            .header(
                AUTHORIZATION,
                format!("OreApi session={}", self.session.session_id),
            )
            .header(header::ACCEPT, "application/json")
    }

    // GET plain request
    async fn get_url(&self, url: String) -> Result<Response> {
        let url = format!("{}{}", self.base_url, url);
        let builder = self.client.get(url);
        let res = self.apply_headers(builder).send().await?;
        self.invalidate().await?;
        Ok(res)
    }

    // GET with String query
    async fn get_url_query(&self, url: String, query: Vec<(String, String)>) -> Result<Response> {
        let url = format!("{}{}", self.base_url, url);
        let builder = self.client.get(url);
        let builder = self.apply_headers(builder);
        let builder = builder.query(&query);
        let res = builder.send().await?;
        self.invalidate().await?;
        Ok(res)
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
            let link = format!("/projects/{}/versions", query.first().unwrap().1);
            self.ore_client.get_url(link).await?
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

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Plugin : {}", self.name)?;
        writeln!(f, "Author : {}", self.namespace.owner)?;
        writeln!(f, "Description : {}", self.description)?;
        writeln!(
            f,
            "Last Updated : {}",
            self.last_updated.parse::<DateTime<Utc>>().unwrap()
        )?;
        writeln!(
            f,
            "Promoted Version : {}",
            self.promoted_versions
                .iter()
                .map(|f| format!(
                    "{} - {}",
                    f.version.clone(),
                    f.tags
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join("-")
                ))
                .collect::<Vec<String>>()
                .join("\n\t| ")
        )?;
        writeln!(f, "{}", self.stats)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OreAuthResponse {
    session: String,
    expires: String,
    r#type: String,
}
