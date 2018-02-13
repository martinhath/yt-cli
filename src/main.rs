extern crate reqwest;
extern crate serde;
extern crate serde_json;

use serde_json::Value;

use std::env::args;

const API_KEY: &'static str = "AIzaSyD-LJtaJMPLkPJ6CjyZcqU9QpU-7Fw0Z_E";

fn req(q: &str) -> reqwest::Response {
    let s = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&part=snippet&q={}",
        API_KEY,
        q
    );
    reqwest::get(&s).unwrap()
}

#[derive(Debug)]
struct Video {
    id: String,
    name: String,
}

fn get_data(body: &str) -> Vec<Video> {
    let v: Value = serde_json::from_str(&body).ok().expect(
        "failed to parse json",
    );
    let _next_page_token = v.get("nextPageToken").and_then(Value::as_str);

    let lst = v.get("items").and_then(Value::as_array).expect(
        "couldn't find item",
    );
    let mut v = Vec::new();
    for o in lst.iter() {
        if o.pointer("/id/kind")
            .and_then(Value::as_str)
            .map(|s| s == "youtube#video")
            .unwrap_or(false)
        {
            v.push(Video {
                id: o.pointer("/id/videoId")
                    .expect("couldn't find /id/videoId")
                    .as_str()
                    .expect("/id/videoId wasn't a string")
                    .to_string(),
                name: o.pointer("/snippet/title")
                    .expect("couldn't find /snippet/title")
                    .as_str()
                    .expect("/snippet/title wasn't a string")
                    .to_string(),
            });
        }
    }
    v
}

fn main() {
    let search = args().skip(1).fold(String::new(), |mut s, e| {
        s.push_str(&e);
        s.push(' ');
        s
    });
    let mut res = req(&search);
    let body = res.text().unwrap();
    let vids = get_data(&body);
    for video in vids.iter() {
        println!("[{}]", video.name);
    }
}
