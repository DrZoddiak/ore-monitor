use std::fmt::Display;

use chrono::{DateTime, Utc};
use reqwest::{
    header::{self, AUTHORIZATION},
    Client, RequestBuilder, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    errors::OreError,
    paginated_project_result::{PaginatedProjectResult, Project},
};

pub struct OreClient {
    client: Client,
    session: OreSession,
    url: String,
}

pub struct OreSession {
    session_id: String,
    expires: String,
}

impl Default for OreSession {
    fn default() -> Self {
        OreSession {
            session_id: "".to_string(),
            expires: "".to_string(),
        }
    }
}

impl OreSession {
    pub fn update(&mut self, response: OreAuthResponse) {
        self.session_id = response.session;
        self.expires = response.expires;
    }
}

pub struct OreAuth {
    ore_client: reqwest::Client,
    ore_session: OreSession,
    url: String,
    api_key: String,
}

//Handles auth for Ore
impl OreAuth {
    pub fn new() -> Self {
        OreAuth {
            ore_client: reqwest::Client::new(),
            ore_session: OreSession::default(),
            url: "https://ore.spongepowered.org/api/v2".to_string(),
            api_key: "beada469-90b5-4f64-b530-7ba3c2e16699".to_string(),
        }
    }

    //Main fn for authorizing
    pub(crate) async fn auth(mut self) -> Result<OreClient, OreError> {
        let res = self.send_request().await;
        let res = self.parse_result(res).await?;
        let res: OreAuthResponse =
            serde_json::from_str(&res).map_err(|e| OreError::SerializationError(e))?;
        self.ore_session.update(res);

        Ok(OreClient::new(self.ore_client, self.ore_session, self.url).await)
    }

    async fn parse_result(&self, res: Result<Response, OreError>) -> Result<String, OreError> {
        res?.text().await.map_err(|e| OreError::ReqwestError(e))
    }

    // Send request for authentication
    async fn send_request(&self) -> Result<Response, OreError> {
        self.ore_client
            .post(format!("{}/authenticate", self.url))
            .header(
                reqwest::header::WWW_AUTHENTICATE,
                format!("OreApi apikey={}", self.api_key),
            )
            .send()
            .await
            .map_err(|e| OreError::ReqwestError(e))
    }
}

type OreResult = Result<(), OreError>;

impl OreClient {
    pub async fn new(client: Client, session: OreSession, url: String) -> Self {
        OreClient {
            client,
            session,
            url,
        }
    }

    fn log(code: StatusCode) {
        let msg = match code {
            StatusCode::NO_CONTENT => "Session Invalidated",
            StatusCode::BAD_REQUEST => "Request not made with a session",
            StatusCode::UNAUTHORIZED => "Api session missing, invalid, or expired",
            StatusCode::FORBIDDEN => "Not enough permission for endpoint",
            _ => "Status code undocumented",
        };
        //println!("Status Code : {}", msg)
    }

    // Invalidates the current session
    pub(crate) async fn invalidate(&self) -> Result<(), OreError> {
        let builder = self.client.delete(format!("{}/sessions/current", self.url));
        let res = self.apply_headers(builder).send().await;
        Self::log(res.map_err(|e| OreError::ReqwestError(e))?.status());
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
    async fn get_url(&self, url: String) -> Result<Response, OreError> {
        let url = format!("{}{}", self.url, url);
        let builder = self.client.get(url);
        let res = self
            .apply_headers(builder)
            .send()
            .await
            .map_err(|e| OreError::ReqwestError(e));
        self.invalidate().await?;
        res
    }

    // GET with String query
    async fn get_url_query(
        &self,
        url: String,
        query: Vec<(String, String)>,
    ) -> Result<Response, OreError> {
        let url = format!("{}{}", self.url, url);
        let builder = self.client.get(url);
        let builder = self.apply_headers(builder);
        let builder = builder.query(&query);
        let res = builder.send().await.map_err(|e| OreError::ReqwestError(e));
        self.invalidate().await?;
        res
    }

    pub(crate) async fn permissions(&mut self) -> Result<(), OreError> {
        let res = self.get_url("/permissions".to_string()).await?;
        res.text().await.map_err(|e| OreError::ReqwestError(e))?;
        Ok(())
    }
}

pub(crate) struct ProjectHandle {
    ore_client: OreClient,
    query: Option<Vec<(String, String)>>,
}

impl ProjectHandle {
    pub async fn new(ore_client: OreClient, query: Option<Vec<(String, String)>>) -> Self {
        ProjectHandle { ore_client, query }
    }

    // Gets projects from query input
    pub(crate) async fn projects(&mut self) -> OreResult {
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

    pub(crate) async fn plugin(&mut self) -> OreResult {
        let res: Response = if let Some(query) = &self.query {
            let link = format!("/projects/{}", query.first().unwrap().1);
            self.ore_client.get_url(link).await?
        } else {
            return Ok(());
        };
        let res: Project = Self::serialize(Self::handle_response(res).await?)?;
        Ok(print!("{}", res))
    }

    // Displays the results for Projects
    fn display_results(result: PaginatedProjectResult) {
        result
            .result
            .iter()
            .for_each(|proj| println!("{}", proj.plugin_id))
    }

    fn serialize<T: DeserializeOwned>(txt: String) -> Result<T, OreError> {
        serde_json::from_str(&txt).map_err(|e| OreError::SerializationError(e))
    }

    // Common method for projects to handle responses.
    async fn handle_response(res: Response) -> Result<String, OreError> {
        Ok(res.text().await.map_err(|e| OreError::ReqwestError(e))?)
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
        writeln!(f, "Statistics : {}", self.stats)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OreAuthResponse {
    session: String,
    expires: String,
    r#type: String,
}
