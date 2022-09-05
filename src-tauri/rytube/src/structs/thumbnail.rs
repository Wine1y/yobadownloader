use serde_json::Value;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Thumbnail{
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>
}

impl Thumbnail {
    pub fn from_json(json: Value) -> Self{
        Thumbnail {
            url: match json.get("url") {
                None => None,
                Some(value) => Some(value.to_string())
            },
            width: match json.get("width") {
                None => None,
                Some(value) => value.to_string().parse::<u32>().ok()
            },
            height: match json.get("height") {
                None => None,
                Some(value) => value.to_string().parse::<u32>().ok()
            }
        }
    }
}