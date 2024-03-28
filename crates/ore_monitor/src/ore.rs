pub mod ore_client {
    use anyhow::Result;
    use reqwest::{
        header::{self, AUTHORIZATION},
        Client, RequestBuilder, Response, StatusCode,
    };
    use tokio_stream::StreamExt;

    use crate::sponge_schemas::OreSession;

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
                    self.session.header_value(),
                )
                .header(AUTHORIZATION, self.session.header_value())
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
    use crate::sponge_schemas::OreSession;

    impl OreSession {
        /// Updates the struct with the new updated values.
        pub fn update(&mut self, response: OreSession) {
            self.session = response.session;
            self.expires = response.expires;
        }

        pub fn header_value(&self) -> String {
            format!("OreApi session={}", self.session)
        }
    }
}

pub mod ore_auth {
    use anyhow::Result;
    use reqwest::Response;
    use std::env;

    use crate::sponge_schemas::OreSession;

    use super::ore_client::OreClient;

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
                api_key: env::var("ORE_API_KEY").expect("ENV_VAR 'ORE_API_KEY' required"),
            }
        }
    }

    /// Handles auth for Ore
    impl OreAuth {
        /// Main method for authorizing, This is also how the [OreClient] is created
        pub async fn auth(mut self) -> Result<OreClient> {
            let res = self.send_request().await;
            let res = res?.text().await?;
            let res: OreSession = serde_json::from_str(&res)?;
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
