#![feature(vec_remove_item)]
extern crate serde_json;

use std::str;
//use std::io::BufRead;

#[macro_use] extern crate chan;

extern crate url;
#[macro_use] extern crate hyper;
#[macro_use] extern crate serde_derive;
extern crate oauthcli;
extern crate tokio_core;
extern crate futures;
extern crate hyper_tls;

use hyper::{Client, Method, Request};
//use std::collections::{HashMap, HashSet};
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::Stream;
//use hyper::client::FutureResponse;
use hyper_tls::HttpsConnector;
//use json_streamer::JsonObjectStreamer;

mod linestream;
use linestream::LineStream;

mod tw;
mod display;
mod commands;

//Change these values to your real Twitter API credentials
static consumer_key: &str = "T879tHWDzd6LvKWdYVfbJL4Su";
static consumer_secret: &str = "OAXXYYIozAZ4vWSmDziI1EMJCKXPmWPFgLbJpB896iIAMIAdpb";
static token: &str = "629126745-Qt6LPq2kR7w58s7WHzSqcs4CIdiue64kkfYYB7RI";
static token_secret: &str = "3BI3YC4WVbKW5icpHORWpsTYqYIj5oAZFkrgyIAoaoKnK";
static lol_auth_token: &str = "641cdf3a4bbddb72c118b5821e8696aee6300a9a";

static STREAMURL: &str = "https://userstream.twitter.com/1.1/user.json?tweet_mode=extended";
static TWEET_LOOKUP_URL: &str = "https://api.twitter.com/1.1/statuses/show.json?tweet_mode=extended";
static USER_LOOKUP_URL: &str = "https://api.twitter.com/1.1/users/show.json";
static ACCOUNT_SETTINGS_URL: &str = "https://api.twitter.com/1.1/account/settings.json";
static GET_FOLLOWER_IDS_URL: &str = "https://api.twitter.com/1.1/followers/ids.json?stringify_ids=true";

header! { (Authorization, "Authorization") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (ContentType, "Content-Type") => [String] }
header! { (Cookie, "cookie") => [String] }


pub struct Queryer {
    client: hyper::client::Client<HttpsConnector<hyper::client::HttpConnector>>,
    core: Core
}

impl Queryer {
    fn do_api_get(&mut self, url: &str) -> Result<serde_json::Value, String> {
        self.issue_request(signed_api_get(url))
    }
    fn do_api_post(&mut self, url: &str) -> Result<serde_json::Value, String> {
        self.issue_request(signed_api_post(url))
    }
    /*
    fn do_web_req(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_web_get(url))
    }*/
    // TODO: make this return the status as well!
    fn issue_request(&mut self, req: hyper::client::Request) -> Result<serde_json::Value, String> {
        let lookup = self.client.request(req);

        let resp: hyper::Response = self.core.run(lookup).unwrap();
        let status = resp.status().clone();

        let chunks: Vec<hyper::Chunk> = self.core.run(resp.body().collect()).unwrap();

        let resp_body: Vec<u8> = chunks.into_iter().flat_map(|chunk| chunk.into_iter()).collect();

        match serde_json::from_slice(&resp_body) {
            Ok(value) => {
                if status != hyper::StatusCode::Ok {
                    Err(format!("!! Requests returned status: {}\n{}", status, value))
                } else {
                    Ok(value)
                }
            }
            Err(e) => {
                Err(format!("!! Requests returned status: {}\nerror deserializing json: {}", status, e))
            }
        }
    }
}

/*
fn signed_web_get(url: &str) -> hyper::client::Request {
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
        headers.set(Cookie(format!("auth_token={}", lol_auth_token)));
        headers.set(Accept("* / *".to_owned()));
        headers.set(ContentType("application/x-www-form-urlencoded".to_owned()));
    };

    req
}
*/

fn signed_api_post(url: &str) -> hyper::client::Request {
    signed_api_req(url, Method::Post)
}

fn signed_api_get(url: &str) -> hyper::client::Request {
    signed_api_req(url, Method::Get)
}

fn signed_api_req(url: &str, method: Method) -> hyper::client::Request {
//    let params: Vec<(String, String)> = vec![("track".to_string(), "london".to_string())];
    let method_string = match method {
        Method::Get => "GET",
        Method::Post => "POST",
        _ => panic!(format!("unsupported method {}", method))
    };

    let params: Vec<(String, String)> = vec![];
    let _param_string: String = params.iter().map(|p| p.0.clone() + &"=".to_string() + &p.1).collect::<Vec<String>>().join("&");

    let header = oauthcli::OAuthAuthorizationHeaderBuilder::new(
        method_string,
        &url::Url::parse(url).unwrap(),
        consumer_key,
        consumer_secret,
        oauthcli::SignatureMethod::HmacSha1,
    )
    .token(token, token_secret)
    .finish();

    let mut req = Request::new(method, url.parse().unwrap());

    {
        let headers = req.headers_mut();
        headers.set(Authorization(header.to_string()));
        headers.set(Accept("*/*".to_owned()));
    };

    req
}

fn main() {

    //Track words
//    let url = "https://stream.twitter.com/1.1/statuses/filter.json";
//    let url = "https://stream.twitter.com/1.1/statuses/sample.json";

    let (ui_tx, mut ui_rx) = chan::sync::<Vec<u8>>(0);

    let mut twete_rx = connect_twitter_stream();

    std::thread::spawn(move || {
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            ui_tx.send(line.into_bytes());
        }
    });

    // I *would* want to load this before spawning the thread, but..
    // tokio_core::reactor::Inner can't be moved between threads safely
    // and beacuse it's an Option-al field, it might be present
    // and rustc says nooooo
    //
    // even though it's not ever present before here
    println!("Loading cache...");

    let mut tweeter = tw::TwitterCache::load_cache();

    println!("Loaded cache!");

    let c2 = Core::new().unwrap(); // i swear this is not where the botnet lives
    let handle = &c2.handle();
    let secondary_connector = HttpsConnector::new(4, handle).unwrap();

    let secondary_client = Client::configure()
        .connector(secondary_connector)
        .build(handle);

    let mut queryer = Queryer {
        client: secondary_client,
        core: c2
    };

    loop {
        match do_ui(ui_rx, twete_rx, &mut tweeter, &mut queryer) {
            Some((new_ui_rx, new_twete_rx)) => {
                ui_rx = new_ui_rx;
                twete_rx = new_twete_rx;
            },
            None => {
                break;
            }
        }
    }
}

fn do_ui(ui_rx_orig: chan::Receiver<Vec<u8>>, twete_rx: chan::Receiver<Vec<u8>>, mut tweeter: &mut tw::TwitterCache, mut queryer: &mut ::Queryer) -> Option<(chan::Receiver<Vec<u8>>, chan::Receiver<Vec<u8>>)> {
    loop {
        let ui_rx_a = &ui_rx_orig;
        let ui_rx_b = &ui_rx_orig;
        chan_select! {
            twete_rx.recv() -> twete => match twete {
                Some(line) => {
                    let jsonstr = std::str::from_utf8(&line).unwrap().trim();
                    /* TODO: replace from_str with from_slice */
                    let json: serde_json::Value = serde_json::from_str(&jsonstr).unwrap();
                    tw::handle_message(json, &mut tweeter, &mut queryer);
                    if tweeter.needs_save && tweeter.caching_permitted {
                        tweeter.store_cache();
                    }
                }
                None => {
                    tweeter.display_info.status("Twitter stream hung up...".to_owned());
                    chan_select! {
                        ui_rx_b.recv() -> input => match input {
                            Some(line) => {
                                if line == "reconnect\n".as_bytes() {
                                    return Some((ui_rx_orig.clone(), connect_twitter_stream()));
                                } else {
                                    tweeter.handle_user_input(line, &mut queryer);
                                }
                            }
                            None => std::process::exit(0)
                        }
                    }
                }
            },
            ui_rx_a.recv() -> user_input => match user_input {
                Some(line) => {
                    tweeter.handle_user_input(line, &mut queryer);
                },
                None => tweeter.display_info.status("UI thread hung up...".to_owned())
            }
            // and then we can introduce a channel that just sends a message every 100 ms or so
            // that acts as a clock!
        }
        // one day display_info should be distinct
        match display::paint(tweeter) {
            Ok(_) => (),
            Err(e) => println!("{}", e)  // TODO: we got here because writing to stdout failed. what to do now?
        };
    }
}

fn url_encode(s: &str) -> String {
    s
        .replace("%", "%25")
        .replace("+", "%2b")
        .replace(" ", "+")
        .replace("\\n", "%0a")
        .replace("\\r", "%0d")
        .replace("\\esc", "%1b")
        .replace("!", "%21")
        .replace("#", "%23")
        .replace("$", "%24")
        .replace("&", "%26")
        .replace("'", "%27")
        .replace("(", "%28")
        .replace(")", "%29")
        .replace("*", "%2a")
        .replace(",", "%2c")
        .replace("-", "%2d")
        .replace(".", "%2e")
        .replace("/", "%2f")
        .replace(":", "%3a")
        .replace(";", "%3b")
        .replace("<", "%3c")
        .replace("=", "%3d")
        .replace(">", "%3e")
        .replace("?", "%3f")
        .replace("@", "%40")
        .replace("[", "%5b")
        .replace("\\", "%5c")
        .replace("]", "%5d")
}

fn connect_twitter_stream() -> chan::Receiver<Vec<u8>> {
    let (twete_tx, twete_rx) = chan::sync::<Vec<u8>>(0);

    std::thread::spawn(move || {
        let mut core = Core::new().unwrap();

        let connector = HttpsConnector::new(1, &core.handle()).unwrap();

        let client = Client::configure()
            .keep_alive(true)
            .connector(connector)
            .build(&core.handle());

        let req = signed_api_get(STREAMURL);
        let work = client.request(req).and_then(|res| {
            let status = res.status();
            if status != hyper::StatusCode::Ok {
                println!("Twitter stream connect was abnormal: {}", status);
                println!("result: {:?}", res);
            }
            LineStream::new(res.body()
                .map(|chunk| futures::stream::iter_ok(chunk.into_iter()))
                .flatten())
                .for_each(|s| {
                    if s.len() != 1 {
                        twete_tx.send(s);
                    };
                    Ok(())
                })
        });

        let resp = core.run(work);
        match resp {
            Ok(_good) => (),
            Err(e) => println!("Error in setting up: {}", e)
        }
    });

    twete_rx
}
