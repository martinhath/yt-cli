extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate console;
extern crate dialoguer;

use serde_json::Value;
use dialoguer::{Input, Select};

use std::process::{Command, Stdio};
use std::io::Write;

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


fn play_video(vid: &Video) {
    let yt_link = format!("https://www.youtube.com/watch?v={}", vid.id);
    let child = Command::new("mpv").arg(&yt_link)
        .stdout(Stdio::piped())
        .spawn().expect(
        "Failed to execute command",
    );
    child.wait_with_output().expect("failed to wait on child");
}

const PROMPT: &str = "[search]";

fn main() {
    let term = console::Term::stdout();
    if let Ok(search_term) = Input::new(PROMPT).interact() {
        let mut res = req(&search_term);
        let body = res.text().unwrap();
        let vids = get_data(&body);

        let mut select = Select::new();
        let strs = vids.iter()
            .map(|v| format!("[{}]", v.name))
            .collect::<Vec<_>>();
        for s in &strs {
            select.item(s);
        }
        if let Ok(i) = select.interact_on(&term) {
            play_video(&vids[i]);
        }
    }
}
