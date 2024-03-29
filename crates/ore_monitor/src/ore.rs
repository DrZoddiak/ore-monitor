pub mod ore_client {
    use super::ore_session::OreSession;
    use anyhow::Result;
    use reqwest::{
        header::{self, AUTHORIZATION},
        Client, RequestBuilder, Response, StatusCode,
    };
    use tokio_stream::StreamExt;

    #[derive(Debug)]
    pub struct OreClient {
        client: Client,
        session: OreSession,
        base_url: String,
    }

    impl OreClient {
        pub async fn new(client: Client, session: OreSession, base_url: String) -> Self {
            OreClient {
                client,
                session,
                base_url,
            }
        }

        fn log_errors(&self, code: StatusCode) {
            let msg = match code {
                // No Content is actually a "successful" error
                StatusCode::NO_CONTENT => None, //Some("Session Invalidated"),
                StatusCode::OK => None,
                StatusCode::BAD_REQUEST => Some("Request not made with a session"),
                StatusCode::UNAUTHORIZED => Some("Api session missing, invalid, or expired"),
                StatusCode::FORBIDDEN => Some("Not enough permission for endpoint"),
                StatusCode::NOT_FOUND => {
                    Some("Resource not found! Ensure you've used the correct identifiers")
                }
                _ => Some("Unexpected Status Code"),
            };
            if let Some(m) = msg {
                println!("Status Error : {}", m)
            }
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
                .header("User-Agent", "Ore-Monitor")
        }

        // Invalidates the current session
        async fn _invalidate(&self) -> Result<()> {
            let builder = self
                .client
                .delete(format!("{}/sessions/current", self.base_url));
            let res = self.apply_headers(builder).send().await?;
            self.log_errors(res.status());
            Ok(())
        }

        pub async fn get_install(
            &self,
            url: String,
            query: Option<Vec<(String, String)>>,
        ) -> Result<Response> {
            let url = "https://ore.spongepowered.org".to_string() + &url;

            let res = self.common_get(url, query).await?;
            // Since this request is not made with the API
            // There is no need to invalidate the request
            // self.invalidate().await?;
            Ok(res)
        }

        pub async fn plugin_responses(&self, id: Vec<String>) -> Result<Vec<String>> {
            let link = id
                .iter()
                .map(|f| format!("/projects/{}", f))
                .collect::<Vec<String>>();

            let mut iter = tokio_stream::iter(link);

            let mut res: Vec<String> = vec![];

            while let Some(v) = iter.next().await {
                let f = self.get(v, None).await?.text().await?;

                res.push(f)
            }

            Ok(res)
        }

        pub async fn get(
            &self,
            url: String,
            query: Option<Vec<(String, String)>>,
        ) -> Result<Response> {
            let url = self.base_url.to_string() + &url;
            let res = self.common_get(url, query).await?;
            self.log_errors(res.status());
            Ok(res)
        }

        // This only exists as a workaround for installs
        async fn common_get(
            &self,
            url: String,
            query: Option<Vec<(String, String)>>,
        ) -> Result<Response> {
            let builder = self.client.get(url);
            let builder = self.apply_headers(builder);

            let builder = if let Some(query) = &query {
                builder.query(&query)
            } else {
                builder
            };

            let res = builder.send().await?;
            Ok(res)
        }
    }
}

mod ore_session {
    use crate::sponge_schemas::ReturnedApiSession;

    /// Represents a session for Ore
    #[derive(Default, Debug)]
    pub(crate) struct OreSession {
        /// The id for the session to pass for auth
        pub session_id: String,
        /// When the session expires
        pub expires: String,
    }

    impl OreSession {
        /// Updates the struct with the new updated values.
        pub fn update(&mut self, response: ReturnedApiSession) {
            self.session_id = response.session;
            self.expires = response.expires;
        }
    }
}

pub mod ore_auth {
    use crate::sponge_schemas::ReturnedApiSession;
    use anyhow::Result;
    use reqwest::Response;
    use std::env;

    use super::{ore_client::OreClient, ore_session::OreSession};

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
                // Retreives key from Env Var or uses a key only capable of viewing public data.
                api_key: env::var("ORE_API_KEY")
                    .unwrap_or("d08a6c8b-3a9e-44c1-9c85-a7dfedba00f5".to_string()),
            }
        }
    }

    /// Handles auth for Ore
    impl OreAuth {
        /// Main method for authorizing, This is also how the [OreClient] is created
        pub async fn auth(mut self) -> Result<OreClient> {
            let res = self.send_request().await;
            let res = res?.text().await?;
            let res: ReturnedApiSession = serde_json::from_str(&res)?;
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
}
