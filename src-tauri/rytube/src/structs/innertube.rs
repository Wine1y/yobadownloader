use std::collections::HashMap;
use serde::Serialize;
use serde_json::{Value, from_str, to_string};
use lazy_static::lazy_static;
use serde_qs;
use reqwest::{Client, Response};


use crate::{Error, unwrap_or_return_error};

#[derive(Serialize)]
struct InnerTubeClient{
    api_key: String,
    context: Context
}

#[derive(Clone, Serialize)]
struct Context{
    client: ClientData
}

#[derive(Clone, Serialize)]
struct ClientData{
    client_name: String,
    client_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_screen: Option<String>
}

impl InnerTubeClient {
    fn new(api_key: &str, client_name: &str, client_version: &str) -> Self{
        InnerTubeClient {
            api_key: api_key.to_owned(),
            context: Context {
                client: ClientData {
                    client_name: client_name.to_owned(),
                    client_version: client_version.to_owned(),
                    client_screen: None
                }
            }
        }
    }

    fn new_embed(api_key: &str, client_name: &str, client_version: &str, client_screen: &str) -> Self{
        InnerTubeClient {
            api_key: api_key.to_owned(),
            context: Context {
                client: ClientData {
                    client_name: client_name.to_owned(),
                    client_version: client_version.to_owned(),
                    client_screen: Some(client_screen.to_owned())
                }
            }
        }
    }
}

lazy_static!{
    static ref DEFAULT_CLIENTS: HashMap<&'static str, InnerTubeClient> = HashMap::from([
        ("WEB",
        InnerTubeClient::new(
                            "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
                            "WEB",
                            "2.20200720.00.02"
                        )
        ),
        ("ANDROID",
        InnerTubeClient::new(
                            "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
                            "ANDROID",
                            "16.20"
                        )
        ),
        ("WEB_EMBED",
        InnerTubeClient::new_embed(
                            "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
                            "WEB",
                            "2.20210721.00.00",
                            "EMBED"
                        )
        ),
        ("ANDROID_EMBED",
        InnerTubeClient::new_embed(
                            "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8",
                            "ANDROID",
                            "16.20",
                            "EMBED"
                        )
        )
    ]);
}



pub(crate) struct InnerTube{
    api_key: String,
    context: Context,
}

impl InnerTube {
    pub(crate) fn new(innertube_client:&str) -> Result<Self, Error>{
        if !DEFAULT_CLIENTS.contains_key(innertube_client){
            return Err(Error::InternalError(format!("InnerTube client is invalid")));
        }
        Ok(
            InnerTube{
                api_key: DEFAULT_CLIENTS[innertube_client].api_key.clone(),
                context: DEFAULT_CLIENTS[innertube_client].context.clone(),
            }
        )
    }

    fn base_url(&self) -> &str{
        "https://www.youtube.com/youtubei/v1"
    }

    fn base_query_params(&self) -> HashMap<&str, String>{
        HashMap::from([
            ("key", self.api_key.clone()),
            ("contentCheckOk", "True".to_string()),
            ("racyCheckOk", "True".to_string())
        ])
    }

    fn base_data(&self) -> Value{
        from_str(&format!("{{\"context\": {}}}", to_string(&self.context).unwrap())).unwrap()
    }

    pub(crate) async fn player(&self, video_id: &str, client: &Client) -> Result<Value, Error>{
        let endpoint = format!("{}/player", self.base_url());
        let mut query = self.base_query_params();
        query.insert("videoId", video_id.to_owned());
        let response = self.call_api(&endpoint, query, Some(self.base_data()), client).await?;
        match response.json().await {
            Err(_) => Err(Error::ParsingError(format!("InnerTube returned invalid json"))),
            Ok(value) => Ok(value)
        }
    }

    async fn call_api(
        &self,
        endpoint: &str,
        query_params: HashMap<&str, String>,
        data: Option<Value>,
        client: &Client
    ) -> Result<Response, Error>{
        let query = unwrap_or_return_error!(
            serde_qs::to_string(&query_params),
            Error::ParsingError(format!("InnerTube query params are invalid"))
        );
        let url = format!("{}?{}", endpoint, query);
        let request = 
            client
            .post(url)
            .header("Content-Type", "application/json")
            .header("accept-language", "en-US,en")
            .header("User-Agent", "Mozilla/5.0");
        let request = match data {
            None => request.header("Content-Length", 0),
            Some(data) => request.json(&data).header("Content-Length", data.to_string().len())
        };
        let response = unwrap_or_return_error!(
            request.send().await,
            Error::HTTPError(format!("Error while sending request to InnerTube API"))
        );
        match response.error_for_status() {
            Ok(response) => Ok(response),
            Err(err) => Err(
                Error::HTTPError(
                    format!("InnerTube API returned {}", err.status().unwrap().as_u16())
                )
            )
        }
    }
}