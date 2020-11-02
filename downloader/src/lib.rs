use dotenv;
use oauth1_request as oauth;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
// use std::fs::File;
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

impl Variant {
    fn _new(b: Option<u32>, c: Option<String>, u: Option<String>) -> Self {
        Variant {
            bitrate: b,
            content_type: c,
            url: u,
        }
    }
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

fn get_urls(v: &Vec<Variant>) -> Vec<String> {
    v.iter()
        .fold(Vec::new(), |mut acc, val| match val.url.clone() {
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

    let urls = get_urls(&x);

    // println!("{:?}", urls);
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

pub fn get_size(urls: &Vec<String>) -> Result<Vec<Input>, Box<dyn Error>> {
    let x: Vec<Input> = urls
        .iter()
        .map(|url| {
            let list_url = Url::parse(&url);

            match list_url {
                Ok(val) => {
                    let xs = val.path_segments().map(|c| c.collect::<Vec<_>>());
                    let size = xs.and_then(|y| {
                        let len = y.len();
                        if len > 4 {
                            Some(y[len - 2].to_string())
                        } else {
                            None
                        }
                    });

                    Input {
                        url: url.to_string(),
                        size,
                    }
                }
                _ => Input {
                    url: url.to_string(),
                    size: None,
                },
            }
        })
        .collect();

    // println!("{:?}", x);
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_tweet_test() {
        let res = get_tweet("1323294043871346688").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_tweet_wrong_id() {
        let res = get_tweet("122890773118001153").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_vid_urls_test() {
        let res = get_vid_urls("1323294043871346688").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_vid_urls_wrong_id() {
        let res = get_vid_urls("122890773118001153").await.unwrap();
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn get_size_test() {
        // let urls = vec![
        // "https://video.twimg.com/ext_tw_video/1134138480253161472/pu/vid/480x480/rLx684w1bBcHv2Tq.mp4?tag=9".to_string(),
        // "https://video.twimg.com/ext_tw_video/1134138480253161472/pu/vid/320x320/-oyV5_rwvgthsq8X.mp4?tag=9".to_string(),
        // "https://video.twimg.com/ext_tw_video/1134138480253161472/pu/vid/640x640/8lpcDipUtTOmeYDl.mp4?tag=9".to_string()
        // ];
        //
        let urls = vec![
            "https://video.twimg.com/amplify_video/1315223083586740225/vid/720x720/EV_bGxB3omHf9UHm.mp4?tag=13".to_string(),
            "https://video.twimg.com/amplify_video/1315223083586740225/vid/480x480/k9mU2z26RXQGKspu.mp4?tag=13".to_string(),
            "https://video.twimg.com/amplify_video/1315223083586740225/vid/320x320/yEq-D0Oy2lcsX68J.mp4?tag=13".to_string()
        ];

        let res = get_size(&urls);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_urls_test() {
        let vars = vec![
            Variant::_new(
                Some(832000),
                Some("video/mp4".to_string()),
                Some("https://video.twimg.com/ext_tw_video/1134138480253161472/pu/vid/480x480/rLx684w1bBcHv2Tq.mp4?tag=9".to_string()),

            )
        ];

        let res = get_urls(&vars);
        assert_eq!(res.len(), vars.len());
    }
}
