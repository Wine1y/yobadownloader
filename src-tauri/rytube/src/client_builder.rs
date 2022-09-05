use reqwest::{Client, header, cookie};
use crate::Error;
use std::sync::Arc;


fn get_cookies() -> cookie::Jar {
    let cookie = "CONSENT=YES+; Path=/; Domain=youtube.com; Secure; Expires=Fri, 01 Jan 2038 00:00:00 GMT;";
    let url = "https://youtube.com".parse().unwrap();

    let jar = reqwest::cookie::Jar::default();
    jar.add_cookie_str(cookie, &url);
    jar
}

fn get_headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();

    headers.insert(header::ACCEPT_LANGUAGE, "en-US,en".parse().unwrap());
    headers.insert(header::USER_AGENT, "Mozilla/5.0".parse().unwrap());
    headers.insert(header::CONNECTION, "keep-alive".parse().unwrap());

    headers
}

pub fn build_client() -> Result<Client, Error>{
    let client = Client::builder()
                 .default_headers(get_headers())
                 .cookie_provider(Arc::new(get_cookies()))
                 .build();
    match client {
        Ok(client) => Ok(client),
        Err(_) => Err(Error::HTTPError(format!("Can't initialize http client")))
    }
}