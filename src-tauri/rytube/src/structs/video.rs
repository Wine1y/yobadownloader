use crate::Error;
use crate::decryption::singature_decryption::Cipher;
use crate::parser::{get_player_and_initial_json, get_webpage, get_base_js_link, get_base_js};
use serde_json::{Value, from_value, from_str};
use super::{Thumbnail, Stream, RawStream};
use reqwest::Client;
use crate::client_builder::build_client;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub struct Video{
    pub id: Option<String>,
    pub title: Option<String>,
    pub desc: Option<String>,
    pub length_seconds: Option<u32>,
    pub author_name: Option<String>,
    pub author_id: Option<String>,
    pub views: Option<String>,
    pub likes: Option<String>,
    pub comments: Option<String>,
    pub is_live: Option<bool>,
    pub streams: Vec<Stream>,
    pub thumbnails: Vec<Thumbnail>,
    pub client: Client
}

impl Video {

    pub fn try_get_id_from_link(link: &str) -> Option<String>{
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(youtube\.com/watch\?v=([a-zA-Z_0-9-]{11}))|(youtu\.be/([a-zA-Z_0-9-]{11}))")
                                    .unwrap();
        }
        match RE.captures(link) {
            None => None,
            Some(captures) => {
                match captures.get(2) {
                    Some(id_match) => Some(id_match.as_str().to_string()),
                    None => match captures.get(4) {
                        Some(id_match) => Some(id_match.as_str().to_string()),
                        None => None
                    }
                }
            } 
        }
    }

    pub async fn from_video_id(id: String) -> Result<Self, Error>{
        Self::from_link(format!("https://www.youtube.com/watch?v={}", id)).await
    }

    pub async fn from_link(link: String) -> Result<Self, Error>{
        let video_id = Video::try_get_id_from_link(&link);
        if video_id.is_none(){
            return Err(Error::ParsingError(format!("Link is invalid: {}", link)));
        }
        let video_id = video_id.unwrap();
        let client = build_client()?;
        let webpage = get_webpage(&client, &link).await?;
        Self::from_webpage(client, webpage, video_id).await
    }

    pub async fn from_webpage(client: Client, webpage: String, id: String) -> Result<Self, Error> {
        let (player_response, initial_data) = get_player_and_initial_json(&id, &webpage, &client).await?;
        let base_js_link = get_base_js_link(&player_response, &webpage);
        let base_js = get_base_js(&client, base_js_link).await;
        let vd = player_response.get("videoDetails");
        if vd.is_none(){
            return Err(Error::ParsingError(format!("Player response is invalid")));
        }
        let vd = vd.unwrap();
        let (likes, views, comments) = parse_likes_views_and_comments(&initial_data);
        Ok(
            Video {
                id: match vd.get("videoId"){
                    None => None,
                    Some(value) => Some(value.as_str().unwrap_or_default().to_owned())
                },
                title: match vd.get("title"){
                    None => None,
                    Some(value) => Some(value.as_str().unwrap_or_default().to_owned())
                },
                desc: match vd.get("shortDescription"){
                    None => None,
                    Some(value) => Some(value.as_str().unwrap_or_default().to_owned())
                },
                length_seconds: match vd.get("lengthSeconds"){
                    None => None,
                    Some(value) => value.as_str().unwrap().replace('"', "").parse::<u32>().ok()
                },
                author_name: match vd.get("author"){
                    None => None,
                    Some(value) => Some(value.as_str().unwrap_or_default().to_owned())
                },
                author_id: match vd.get("channelId"){
                    None => None,
                    Some(value) => Some(value.as_str().unwrap_or_default().to_owned())
                },
                views,
                likes,
                comments,
                is_live: match vd.get("isLiveContent"){
                    None => None,
                    Some(value) => value.as_bool()
                },
                streams: match get_streams(&player_response, base_js) {
                    None => vec![],
                    Some(streams) => streams
                },
                client: client,
                thumbnails: match vd.get("thumbnail"){
                    None => vec![],
                    Some(value) => match value.get("thumbnails"){
                        None => vec![],
                        Some(value) => match value.as_array(){
                            None => vec![],
                            Some(value) => value
                                           .to_owned()
                                           .into_iter()
                                           .map(|json_thumbnail| Thumbnail::from_json(json_thumbnail))
                                           .collect::<Vec<Thumbnail>>()
                        }
                    }
                },
            }
        )
    }
}

fn parse_likes_views_and_comments(initial_data: &Value) -> (Option<String>, Option<String>, Option<String>){
    (parse_likes(initial_data), parse_views(initial_data), parse_comments(initial_data))
}

fn parse_likes(initial_data: &Value) -> Option<String>{
    let likes_label = initial_data
        .get("contents")?
        .get("twoColumnWatchNextResults")?
        .get("results")?
        .get("results")?
        .get("contents")?
        .get(0)?
        .get("videoPrimaryInfoRenderer")?
        .get("videoActions")?
        .get("menuRenderer")?
        .get("topLevelButtons")?
        .get(0)?
        .get("segmentedLikeDislikeButtonRenderer")?
        .get("likeButton")?
        .get("toggleButtonRenderer")?
        .get("defaultText")?
        .get("accessibility")?
        .get("accessibilityData")?
        .get("label")?
        .as_str()?
        .replace('\u{00a0}', " ");
    Some(likes_label)
}

fn parse_views(initial_data: &Value) -> Option<String>{
    let views_label = initial_data
        .get("contents")?
        .get("twoColumnWatchNextResults")?
        .get("results")?
        .get("results")?
        .get("contents")?
        .get(0)?
        .get("videoPrimaryInfoRenderer")?
        .get("viewCount")?
        .get("videoViewCountRenderer")?
        .get("viewCount")?
        .get("simpleText")?
        .as_str()?
        .replace('\u{00a0}', " ");
    Some(views_label)
}

fn parse_comments(initial_data: &Value) -> Option<String>{
    let comments_label = initial_data
        .get("contents")?
        .get("twoColumnWatchNextResults")?
        .get("results")?
        .get("results")?
        .get("contents")?
        .get(2)?
        .get("itemSectionRenderer")?
        .get("contents")?
        .get(0)?
        .get("commentsEntryPointHeaderRenderer")?
        .get("commentCount")?
        .get("simpleText")?
        .as_str()?
        .replace('\u{00a0}', " ");
    Some(comments_label)
}

fn get_streams(player_response: &Value, base_js: Option<String>) -> Option<Vec<Stream>>{
    let streaming_data = player_response.get("streamingData")?;
    let empty: Value = from_str("[]").unwrap();
    let cipher: Option<Cipher> = match base_js {
        None => None,
        Some(js) => Cipher::from_js(&js).ok()
    };
    let formats = streaming_data.get("formats")
        .unwrap_or(&empty)
        .as_array()
        .unwrap()
        .into_iter();
    let adaptive_formats = streaming_data.get("adaptiveFormats")
        .unwrap_or(&empty)
        .as_array()
        .unwrap()
        .into_iter();

    Some(
        formats
        .chain(adaptive_formats)
        .filter_map(|value| {
            let raw_stream: Result<RawStream, serde_json::Error> = from_value(value.clone());
            match raw_stream{
                Err(_) => None,
                Ok(raw) => match Stream::from_raw_stream(raw, &cipher) {
                    Ok(stream) => Some(stream),
                    Err(_) => None
                },
                
            }
        })
        .collect()
    )
}