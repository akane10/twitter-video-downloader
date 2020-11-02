#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use downloader::*;
use rocket::request::Request;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
// use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;

#[tokio::main]
async fn whatever(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let id = get_id(url)?;
    let res = match id {
        Some(val) => get_vid_urls(&val).await?,
        _ => vec![],
    };

    println!("{:?}", res);
    Ok(res)
}

#[post("/", data = "<input>")]
fn index_api(input: Json<Input>) -> Result<Json<Value>, Box<dyn Error>> {
    let data: Vec<String> = whatever(&input.url)?;
    let res = get_size(&data)?;
    Ok(Json(json!(res)))
}

#[catch(500)]
fn internal_error() -> &'static str {
    "Whoops! Looks like we messed up."
}
#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Couldn't find '{}'. Try something else?", req.uri())
}
pub fn rocket_app() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .mount("/api", routes![index_api])
        .register(catchers![not_found, internal_error])
}
