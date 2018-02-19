extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate console;
extern crate dialoguer;

use serde_json::Value;
use dialoguer::{Input, Select};

use std::process::{Command, Stdio};

const API_KEY: &'static str = "AIzaSyD-LJtaJMPLkPJ6CjyZcqU9QpU-7Fw0Z_E";

fn req(ctx: &Context, q: &str) -> reqwest::Response {
    let mut s = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&part=snippet&q={}",
        API_KEY,
        q
    );
    if let Some(ref t) = ctx.next_page_token {
        s.push_str("&pageToken=");
        s.push_str(t);
    }
    reqwest::get(&s).unwrap()
}

#[derive(Debug)]
struct Video {
    id: String,
    name: String,
}

struct Context {
    select_i: usize,
    videos: Vec<Video>,
    next_page_token: Option<String>,
}

fn get_data(ctx: &mut Context, body: &str) -> Vec<Video> {
    let v: Value = serde_json::from_str(&body).ok().expect(
        "failed to parse json",
    );
    let next_page_token = v.get("nextPageToken").and_then(Value::as_str).map(
        ::std::string::ToString::to_string,
    );
    ctx.next_page_token = next_page_token;

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
    let child = Command::new("mpv")
        .arg(&yt_link)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");
    child.wait_with_output().expect("failed to wait on child");
}

const PROMPT: &str = "[search]";

fn main() {
    let term = console::Term::stdout();
    let mut ctx = Context {
        select_i: 0,
        videos: vec![],
        next_page_token: None,
    };
    if let Ok(search_term) = Input::new(PROMPT).interact() {
        let mut select;
        loop {
            let mut res = req(&ctx, &search_term);
            let body = res.text().unwrap();
            let vids = get_data(&mut ctx, &body);
            ctx.videos.extend(vids.into_iter());

            let strs = ctx.videos
                .iter()
                .map(|v| format!("[{}]", v.name))
                .collect::<Vec<_>>();
            select = Select::new();
            for s in &strs {
                select.item(s);
            }
            select.item("+More");
            select.default(ctx.select_i);
            if let Ok(i) = select.interact_on(&term) {
                if i == ctx.videos.len() {
                    // more was clicked. We have saved the stuff in ctx. Just loop.
                    ctx.select_i = i;
                } else {
                    play_video(&ctx.videos[i]);
                    break;
                }
            }
        }
    }
}
