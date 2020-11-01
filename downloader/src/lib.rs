use oauth1_request as oauth;
// use std::fs::File;
// use reqwest::multipart;
use dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use url::Url;

type Hjson = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub url: String,
    pub size: Option<String>,
}

#[derive(oauth::Request, Deserialize, Serialize)]
struct Param {
    id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Variant {
    bitrate: Option<u32>,
    content_type: Option<String>,
    url: Option<String>,
}

async fn get_tweet(id: &str) -> Result<Hjson, Box<dyn Error>> {
    let uri = "https://api.twitter.com/1.1/statuses/show.json";

    let consumer_key: &str = &dotenv::var("CONSUMER_KEY").unwrap();
    let consumer_secret: &str = &dotenv::var("CONSUMER_SECRET").unwrap();
    let access_token: &str = &dotenv::var("ACCESS_TOKEN").unwrap();
    let access_token_secret: &str = &dotenv::var("ACCESS_TOKEN_SECRET").unwrap();

    let req = Param { id: id.to_string() };
    let client = reqwest::Client::new();

    let token = oauth::Token::from_parts(
        consumer_key,
        consumer_secret,
        access_token,
        access_token_secret,
    );

    let authorization_header_stat = oauth::get(uri, &req, &token, oauth::HmacSha1);
    let uri = oauth::to_uri_query(uri.to_owned(), &req);
    let res = client
        .get(&uri)
        .header("Authorization", authorization_header_stat.clone())
        .send()
        .await?
        .json::<Hjson>()
        .await?;

    // let file = File::create("example-data.json")?;
    // let _data = serde_json::to_writer_pretty(file, &res)?;
    // extended_entities -> media -> video_info / type
    // println!("{:?}", res.get("extended_entities"));
    Ok(res)
}

fn get_urls(v: Vec<Variant>) -> Vec<String> {
    v.into_iter()
        .fold(Vec::new(), |mut acc, val| match val.url {
            Some(v) => {
                if v.contains("mp4") {
                    acc.push(v);
                }
                acc
            }
            _ => acc,
        })
}

// #[tokio::main]
pub async fn get_vid_urls(id: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let res = get_tweet(id).await?;
    let variants = res.get("extended_entities").and_then(|x| {
        let val = x["media"][0]["video_info"]["variants"].clone();
        match val {
            Value::Null => None,
            _ => Some(val),
        }
    });

    let x: Vec<Variant> = match variants {
        Some(val) => serde_json::from_value(val).unwrap_or(Vec::new()),
        _ => Vec::new(),
    };

    // println!("{:?}", get_urls(x));
    let urls = get_urls(x);
    Ok(urls)
}

pub fn get_id(url: &str) -> Result<Option<String>, Box<dyn Error>> {
    let x = Url::parse(url)?;
    let vec = x
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .unwrap_or(vec![]);

    let l = vec.len();
    if l > 1 {
        Ok(Some(vec[l - 1].to_string()))
    } else {
        Ok(None)
    }
}

pub fn get_size(urls: Vec<String>) -> Result<Vec<Input>, Box<dyn Error>> {
    let x: Vec<Input> = urls
        .into_iter()
        .map(|url| {
            let list_url = Url::parse(&url);

            match list_url {
                Ok(val) => {
                    let xs = val.path_segments().map(|c| c.collect::<Vec<_>>());
                    let size = xs.and_then(|y| {
                        let len = y.len();
                        if len > 5 {
                            Some(y[len - 2].to_string())
                        } else {
                            None
                        }
                    });

                    Input { url, size }
                }
                _ => Input { url, size: None },
            }
        })
        .collect();

    Ok(x)
}
