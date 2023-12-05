use crate::{Error, unwrap_or_return_error};
use crate::structs::innertube::InnerTube;
use serde_json::Value;
use lazy_static::lazy_static;
use reqwest::Client;
use regex::Regex;

pub async fn get_webpage(client: &Client, link: &str) -> Result<String, Error>{
    let res = unwrap_or_return_error!(
        client.get(link).send().await,
        Error::HTTPError(format!("Can't access {}", link))
    );
    let res = res.error_for_status();
    if res.is_err(){
        return Err(Error::HTTPError(format!("Video link returned {}", res.unwrap_err().status().unwrap().as_u16())));
    }
    let res = res.unwrap();
    let webpage = unwrap_or_return_error!(
        res.text().await,
        Error::ParsingError(format!("Can't get html of {}", link))
    );
    Ok(webpage)
}

async fn get_innertube_player_response(video_id: &str, client: &Client) -> Option<Value>{
    let innertube = InnerTube::new("ANDROID").unwrap();
    innertube.player(video_id, client).await.ok()
}

fn parse_json_player_response(webpage: &str) -> Option<Value>{
    let json_start_index = webpage.find("var ytInitialPlayerResponse = ")?
        +"var ytInitialPlayerResponse = ".len();
    let json_end_index = &webpage[json_start_index..].find("</script>")?
        +json_start_index-1;
    let json_data: Value = serde_json::from_str(&webpage[json_start_index..json_end_index]).ok()?;
    Some(json_data)
}

fn parse_json_initial_data(webpage: &str) -> Option<Value>{
    let json_start_index = webpage.find("var ytInitialData = ")?
        +"var ytInitialData = ".len();
    let json_end_index = &webpage[json_start_index..].find("</script>")?
        +json_start_index-1;
    let json_data: Value = serde_json::from_str(&webpage[json_start_index..json_end_index]).ok()?;
    Some(json_data)
}

pub fn get_base_js_link(player_response: &Value, webpage: &str) -> Option<String>{
    if player_response.get("assets").is_some(){
        let assets =  player_response.get("assets").unwrap();
        if assets.get("js").is_some(){
            return Some(format!("https://youtube.com{}", assets.get("js").unwrap().to_string()));
        }
    }
    lazy_static!{
        static ref RE: Regex = Regex::new(r"(/s/player/[\w\d]+/[\w\d_/.]+/base\.js)").unwrap();
    }
    match RE.captures(webpage) {
        Some(base_js_url) => Some(format!("https://youtube.com{}", base_js_url.get(1).unwrap().as_str().to_string())),
        None => None
    }
}

pub async fn get_base_js(client: &Client, link: Option<String>) -> Option<String>{
    let link = match link {
        Some(link_string) => link_string,
        None => {return None;}
    };
    let res = client.get(link).send().await.ok()?;
    let res = res.error_for_status().ok()?;
    Some(res.text().await.ok()?)
}

pub async fn get_player_and_initial_json(video_id: &str,  webpage: &str, client: &Client) -> Result<(Value, Value), Error>{
    let player_response = match get_innertube_player_response(&video_id, client).await{
        Some(response) => {
            if response.get("videoDetails").is_some() && response.get("streamingData").is_some(){
                Some(response)
            }else{
                parse_json_player_response(&webpage)
            }
        },
        None => parse_json_player_response(&webpage)
    };
    let initial_data = parse_json_initial_data(&webpage);
    if player_response.is_some() && initial_data.is_some(){
        Ok((player_response.unwrap(), initial_data.unwrap()))
    }else {
        Err(Error::ParsingError(format!("Webpage is invalid")))
    }
}

