extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate console;
extern crate dialoguer;

use serde_json::Value;
use dialoguer::{Input, Select};

use std::process::{Command, Stdio};
use std::env;

const PROMPT: &str = "[search]";

fn req(ctx: &Context, q: &str) -> reqwest::Response {
    let mut s = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&part=snippet&q={}&maxResults=20",
        ctx.api_key,
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
    title: String,
    channel: String,
}

fn video_to_string(v: &Video) -> String {
    format!(" {}: {}", v.channel, v.title)
}

struct Context {
    select_i: usize,
    api_key: String,
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
        "couldn't find items",
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
                title: o.pointer("/snippet/title")
                    .expect("couldn't find /snippet/title")
                    .as_str()
                    .expect("/snippet/title wasn't a string")
                    .to_string(),
                channel: o.pointer("/snippet/channelTitle")
                    .expect("couldn't find /snippet/channelTitle")
                    .as_str()
                    .expect("/snippet/channelTitle wasn't a string")
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

fn main() {
    let api_key = match env::var("YT_API_KEY") {
        Ok(s) => s,
        Err(_) => panic!("Set the env variable 'YT_API_KEY' to be the API key."),
    };
    let term = console::Term::stdout();
    let mut ctx = Context {
        select_i: 0,
        videos: vec![],
        next_page_token: None,
        api_key,
    };
    if let Ok(search_term) = Input::new(PROMPT).interact() {
        let mut select;
        loop {
            let mut res = req(&ctx, &search_term);
            let body = res.text().unwrap();
            let vids = get_data(&mut ctx, &body);
            ctx.videos.extend(vids.into_iter());

            let strs = ctx.videos.iter().map(video_to_string);
            select = Select::new();
            for s in strs {
                select.item(&s);
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
