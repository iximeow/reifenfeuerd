extern crate serde_json;

use rustc_serialize::json::Json;
use std::str;
use std::io::BufRead;

extern crate url;
#[macro_use] extern crate hyper;
extern crate rustc_serialize;
extern crate oauthcli;
extern crate tokio_core;
extern crate futures;
extern crate hyper_tls;

use hyper::{Client, Method, Request};
use hyper::header::Headers;
use std::io::BufReader;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::Stream;
use hyper::client::FutureResponse;
use hyper_tls::HttpsConnector;
//use json_streamer::JsonObjectStreamer;

//Change these values to your real Twitter API credentials
static consumer_key: &str = "0af9c1AoEi5X7IjtOKAtP60Za";
static consumer_secret: &str = "1fxEzRhQtQSWKus4oqDwdg5DALIjGpINg0PGjkYVwKT8EEMFCh";
static token: &str = "629126745-VePBD9ciKwpuVuIeEcNnxwxQFNWDXEy8KL3dGRRg";
static token_secret: &str = "uAAruZzJu03NvMlH6cTeGku7NqVPro1ddKN4BxORy5hWG";

static streamurl: &str = "https://userstream.twitter.com/1.1/user.json?tweet_mode=extended";
static tweet_lookup_url: &str = "https://api.twitter.com/1.1/statuses/show.json";
static user_lookup_url: &str = "https://api.twitter.com/1.1/users/lookup.json";

header! { (Authorization, "Authorization") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (ContentType, "Content-Type") => [String] }

fn render_tweet(structure: serde_json::Map<String, serde_json::Value>, client: &hyper::client::Client<HttpsConnector<hyper::client::HttpConnector>>, c2: &mut Core) {
    if structure.contains_key("event") {
        match &structure["event"].as_str().unwrap() {
            &"follow" => println!("followed! by {} (@{})", structure["source"]["name"].as_str().unwrap(), structure["source"]["screen_name"].as_str().unwrap()),
            &"favorite" => println!("{} fav'd", structure["source"]["screen_name"].as_str().unwrap()),
            &"favorited_retweet" => println!("{} fav'd rt", structure["source"]["screen_name"].as_str().unwrap()),
            e => println!("unrecognized event: {}", e)
        }
    } else if structure.contains_key("delete") {
        println!("delete...");
        let deleted_user_id = structure["delete"]["status"]["user_id_str"].as_str().unwrap();
        let userjson = look_up_user(deleted_user_id, client, c2);
        let screen_name = match userjson {
            Some(ref json) => {
                json[0]["screen_name"].as_str().clone().unwrap()
            },
            None => { "idk lol" }
        };
        println!("who? {} - {}", structure["delete"]["status"]["user_id_str"], screen_name);
    } else if structure.contains_key("user") && structure.contains_key("id") {
        // probably a tweet
        let mut twete: &serde_json::Map<String, serde_json::Value> = &structure; // type isn't actually necessary here, but that lead me to the right rvalue
        let source_name = twete["user"]["name"].as_str().unwrap();
        let source_screen_name = twete["user"]["screen_name"].as_str().unwrap();
        if twete.contains_key("retweeted_status") {
            // render RT, actually
            twete = twete["retweeted_status"].as_object().unwrap();

            let author_name = twete["user"]["name"].as_str().unwrap();
            let author_screen_name = twete["user"]["screen_name"].as_str().unwrap();

            println!("{} (@{}) via {} (@{}) RT:                           https://twitter.com/i/web/status/{}", author_name, author_screen_name, source_name, source_screen_name, twete["id_str"].as_str().unwrap());
        } else {
            println!("{} (@{})                                            https://twitter.com/i/web/status/{}", source_name, source_screen_name, twete["id_str"].as_str().unwrap());
        }

        let twete_text = full_twete_text(twete);

        println!("{}", twete_text); // to remove the link for quoted twetes, gotta look for: "is the last url the same as the url for the quoted id?" and if so s///

        if twete.contains_key("quoted_status") {
            let quoted = twete["quoted_status"].as_object().unwrap();
            println!(
                "  {} (@{})                                             https://twitter.com/i/web/status/{}\n    {}",
                quoted["user"]["name"].as_str().unwrap(),
                quoted["user"]["screen_name"].as_str().unwrap(),
                quoted["id_str"].as_str().unwrap(),
                full_twete_text(quoted).split("\n").collect::<Vec<&str>>().join("\n    ")
            );
//            println!(" and it's a quote ");
        }
    }
    println!("-------------------------------------------------------");
    println!("");
}

fn full_twete_text(twete: &serde_json::map::Map<String, serde_json::Value>) -> String {
//    let looked_up;
    let mut twete_text: String;
    twete_text = if twete["truncated"].as_bool().unwrap() {
        twete["extended_tweet"]["full_text"].as_str().unwrap().to_string()
//        looked_up = look_up_tweet(twete["id_str"].as_str().unwrap()).unwrap();
//        looked_up["text"].as_str().unwrap().to_string()
    } else {
        twete["text"].as_str().unwrap().to_string()
    };

    twete_text = twete_text
        .replace("&amp;", "&")
        .replace("&gt;", ">")
        .replace("&lt;", "<");

    for url in twete["entities"]["urls"].as_array().unwrap() {
        if url["expanded_url"].as_str().unwrap().len() < 200 {
            twete_text = twete_text.replace(url["url"].as_str().unwrap(), url["expanded_url"].as_str().unwrap());
        }
    }

    twete_text
}

fn signed_get(url: &str) -> hyper::client::Request {
//    let params: Vec<(String, String)> = vec![("track".to_string(), "london".to_string())];
    let params: Vec<(String, String)> = vec![];
    let param_string: String = params.iter().map(|p| p.0.clone() + &"=".to_string() + &p.1).collect::<Vec<String>>().join("&");

    let header = oauthcli::authorization_header(
        "GET",
        url::Url::parse(url).unwrap(),
        None, // Realm
        consumer_key,
        consumer_secret,
        Some(token),
        Some(token_secret),
        oauthcli::SignatureMethod::HmacSha1,
        &oauthcli::timestamp(),
        &oauthcli::nonce(),
        None, // oauth_callback
        None, // oauth_verifier
        params.clone().into_iter()
    );

    let mut req = Request::new(Method::Get, url.parse().unwrap());

    req.set_body(param_string);

    {
        let mut headers = req.headers_mut();
        headers.set(Authorization(header.to_owned()));
        headers.set(Accept("*/*".to_owned()));
        headers.set(ContentType("application/x-www-form-urlencoded".to_owned()));
    };

    req
}

fn look_up_user(id: &str, client: &hyper::client::Client<HttpsConnector<hyper::client::HttpConnector>>, core: &mut Core) -> Option<serde_json::Value> {
    let lookup = client.request(signed_get(&format!("{}?user_id={}", user_lookup_url, id)));

    let resp: hyper::Response = core.run(lookup).unwrap();

    let chunks: Vec<hyper::Chunk> = core.run(resp.body().collect()).unwrap();

    let resp_body: Vec<u8> = chunks.into_iter().flat_map(|chunk| chunk.into_iter()).collect();

    match serde_json::from_slice(&resp_body) {
        Ok(value) => Some(value),
        Err(e) => {
            println!("error deserializing json: {}", e);
            None
        }
    }
}

fn look_up_tweet(id: &str, client: &hyper::client::Client<HttpsConnector<hyper::client::HttpConnector>>, core: &mut Core) -> Option<serde_json::Value> {
    let lookup = client.request(signed_get(&format!("{}?id={}", tweet_lookup_url, id)));

    let resp: hyper::Response = core.run(lookup).unwrap();

    let chunks: Vec<hyper::Chunk> = core.run(resp.body().collect()).unwrap();

    let resp_body: Vec<u8> = chunks.into_iter().flat_map(|chunk| chunk.into_iter()).collect();

    match serde_json::from_slice(&resp_body) {
        Ok(value) => Some(value),
        Err(e) => {
            println!("error deserializing json: {}", e);
            None
        }
    }
}


fn main() {


    //Track words
//    let url = "https://stream.twitter.com/1.1/statuses/filter.json";
//    let url = "https://stream.twitter.com/1.1/statuses/sample.json";

    let mut core = Core::new().unwrap();

    let connector = HttpsConnector::new(1, &core.handle()).unwrap();

    let client = Client::configure()
        .keep_alive(true)
        .connector(connector)
        .build(&core.handle());

    let req = signed_get(streamurl);

    println!("starting!");
//    println!("lookup'd");

//    println!("requesting...");
    /*
    let work = client.request(req).and_then(|res| {
        res.body().for_each(move |body: hyper::Chunk| {
            println!("hmmm");
            println!("{}", std::str::from_utf8(&body).unwrap());
            Ok(())
        })
    });
    */

    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

    let remote = core.remote();

    std::thread::spawn(move || {
        let mut c2 = Core::new().unwrap(); // i swear this is not where the botnet lives
        let handle = &c2.handle();
        let secondaryConnector = HttpsConnector::new(4, handle).unwrap();

        let secondaryClient = Client::configure()
            .connector(secondaryConnector)
            .build(handle);
        loop {
            match rx.recv() {
                Ok(line) => {
                    let jsonstr = std::str::from_utf8(&line).unwrap().trim();
                    //println!("{}", jsonstr);
                    let json: serde_json::Value = serde_json::from_str(&jsonstr).unwrap();
                    match json {
                        serde_json::Value::Object(objmap) => render_tweet(objmap, &secondaryClient, &mut c2),
                        f => println!("Unexpected object: {}", f)
                    }
                }
                Err(e) => { println!("{}", e); }
            }
        }
    });

    println!("Before?");
    let work = client.request(req).and_then(|res| {
        LineStream::new(res.body()
            .map(|chunk| futures::stream::iter(chunk.into_iter().map(|b| Ok(b))))
            .flatten())
            .for_each(|s| {
                if s.len() != 1 {
//                    println!("Send!: {}", std::str::from_utf8(&s).unwrap());
                    tx.send(s);
                };
                Ok(())
            })
    });

    let resp = core.run(work).unwrap();
    println!("After?");

    /*
    for obj in BufReader::new(res).json_objects() {
        println!("{:?}", obj.as_object().unwrap().get("text").unwrap().as_string().unwrap());
    }*/

}

//extern crate futures;
//use futures::stream::Stream;
//use futures::{Future, Poll, Async};
use futures::{Poll, Async};
/*
fn main() {
  let lines = "line 1.\nline 2...\n   LINE 3  \n".as_bytes();
  let bytestream = futures::stream::iter(lines.iter().map(|byte| -> Result<_, ()> { Ok(*byte) }));
  let linestream = LineStream::new(bytestream);
  
  linestream.for_each(|line| {
    println!("Bytes: {:?}", line);
    println!("Line: {}", String::from_utf8(line).unwrap());
    Ok(())
  }).wait().unwrap()
}
*/

struct LineStream<S, E> where S: Stream<Item=u8, Error=E> {
  stream: S,
  progress: Vec<u8>
}

impl<S,E> LineStream<S, E> where S: Stream<Item=u8, Error=E> + Sized {
  pub fn new(stream: S) -> LineStream<S, E> {
    LineStream {
      stream: stream,
      progress: vec![]
    }
  }
}

impl<S, E> Stream for LineStream<S, E> where S: Stream<Item=u8, Error=E> {
  type Item = Vec<u8>;
  type Error = E;
  
  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    loop {
      match self.stream.poll() {
        Ok(Async::Ready(Some(byte))) => {
          if byte == 0x0a { 
            let mut new_vec = vec![];
            std::mem::swap(&mut self.progress, &mut new_vec);
            return Ok(Async::Ready(Some(new_vec)))
          } else {
            self.progress.push(byte)
          }
        },
        Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
        Ok(Async::NotReady) => return Ok(Async::NotReady),
        Err(e) => return Err(e)
      }
    }
  }
}

