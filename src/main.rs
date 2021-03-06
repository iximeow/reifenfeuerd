extern crate serde_json;

extern crate chrono;
extern crate termion;
extern crate termios;

use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

use termion::input::TermRead;
use termion::event::{Event, Key};

use std::str;
//use std::io::BufRead;
use std::io::stdin;

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
    fn do_api_get_noparam(&mut self, url: &str, app_cred: &tw::Credential, user_cred: &tw::Credential) -> Result<serde_json::Value, String> {
        self.do_api_get(url, &vec![], app_cred, user_cred)
    }

    fn do_api_get(&mut self, url: &str, params: &Vec<(&str, &str)>, app_cred: &tw::Credential, user_cred: &tw::Credential) -> Result<serde_json::Value, String> {
        self.issue_request(signed_api_get(url, params, app_cred, user_cred))
    }

    fn do_api_post_noparam(&mut self, url: &str, app_cred: &tw::Credential, user_cred: &tw::Credential) -> Result<serde_json::Value, String> {
        self.do_api_post(url, &vec![], app_cred, user_cred)
    }

    fn do_api_post(&mut self, url: &str, params: &Vec<(&str, &str)>, app_cred: &tw::Credential, user_cred: &tw::Credential) -> Result<serde_json::Value, String> {
        self.issue_request(signed_api_post(url, params, app_cred, user_cred))
    }
    /*
    fn do_web_req(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_web_get(url))
    }*/
    // TODO: make this return the status as well!
    fn issue_request(&mut self, req: hyper::client::Request) -> Result<serde_json::Value, String> {
        let resp_body = self.raw_issue_request(req);
        resp_body.and_then(|body| serde_json::from_slice(&body).map_err(|e| e.to_string()))
    }
    fn raw_issue_request(&mut self, req: hyper::client::Request) -> Result<Vec<u8>, String> {
        let lookup = self.client.request(req);

        let resp: hyper::Response = self.core.run(lookup).unwrap();
        let status = resp.status().clone();

        let chunks: Vec<hyper::Chunk> = self.core.run(resp.body().collect()).unwrap();

        let resp_body: Vec<u8> = chunks.into_iter().flat_map(|chunk| chunk.into_iter()).collect();
        if status != hyper::StatusCode::Ok {
            Err(format!("!! Requests returned status: {} - {:?}", status, std::str::from_utf8(&resp_body)))
        } else {
            Ok(resp_body)
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

fn signed_api_post(url: &str, params: &Vec<(&str, &str)>, app_cred: &tw::Credential, user_cred: &tw::Credential) -> hyper::client::Request {
    signed_api_req_with_token(url, params, Method::Post, app_cred, user_cred)
}

fn signed_api_get(url: &str, params: &Vec<(&str, &str)>, app_cred: &tw::Credential, user_cred: &tw::Credential) -> hyper::client::Request {
    signed_api_req_with_token(url, params, Method::Get, app_cred, user_cred)
}

fn signed_api_req_with_token(url: &str, params: &Vec<(&str, &str)>, method: Method, app_cred: &tw::Credential, user_cred: &tw::Credential) -> hyper::client::Request {
    inner_signed_api_req(url, params, method, app_cred, Some(user_cred))
}

fn signed_api_req_no_params(url: &str, method: Method, app_cred: &tw::Credential) -> hyper::client::Request {
    inner_signed_api_req(url, &vec![], method, app_cred, None)
}

fn signed_api_req(url: &str, params: &Vec<(&str, &str)>, method: Method, app_cred: &tw::Credential) -> hyper::client::Request {
    inner_signed_api_req(url, params, method, app_cred, None)
}

fn inner_signed_api_req(url: &str, params: &Vec<(&str, &str)>, method: Method, app_cred: &tw::Credential, maybe_user_cred: Option<&tw::Credential>) -> hyper::client::Request {
//    let params: Vec<(String, String)> = vec![("track".to_string(), "london".to_string())];
    let method_string = match method {
        Method::Get => "GET",
        Method::Post => "POST",
        _ => panic!(format!("unsupported method {}", method))
    };

    let escaped = params.iter().map(|&(ref k, ref v)| format!("{}={}",
        url_encode(k),
        url_encode(v)
    ));
    let params_str = escaped.collect::<Vec<String>>().join("&");

    let constructed_url = if params_str.len() > 0 {
        format!("{}?{}", url, params_str)
    } else {
        url.to_owned()
    };

    let parsed_url = url::Url::parse(&constructed_url).unwrap();

    let mut builder = oauthcli::OAuthAuthorizationHeaderBuilder::new(
        method_string,
        &parsed_url,
        app_cred.key.to_owned(),
        app_cred.secret.to_owned(),
        oauthcli::SignatureMethod::HmacSha1,
    );

    if let Some(user_cred) = maybe_user_cred {
        builder.token(user_cred.key.to_owned(), user_cred.secret.to_owned());
    }

    let header = builder.finish();

    let mut req = Request::new(method, constructed_url.parse().unwrap());

    {
        let headers = req.headers_mut();
        headers.set(Authorization(header.to_string()));
        headers.set(Accept("*/*".to_owned()));
    };

    req
}

static mut connection_id: u8 = 0;

fn get_id() -> u8 {
    unsafe {
        let curr_id = connection_id;
        connection_id += 1;
        curr_id
    }
}

fn main() {

    //Track words
//    let url = "https://stream.twitter.com/1.1/statuses/filter.json";
//    let url = "https://stream.twitter.com/1.1/statuses/sample.json";

    let (ui_tx, mut ui_rx) = chan::sync::<Result<termion::event::Event, std::io::Error>>(0);

    // I *would* want to load this before spawning the thread, but..
    // tokio_core::reactor::Inner can't be moved between threads safely
    // and beacuse it's an Option-al field, it might be present
    // and rustc says nooooo
    //
    // even though it's not ever present before here
    println!("Loading cache...");

    let mut display_info = display::DisplayInfo::default();

    let mut tweeter = tw::TwitterCache::load_cache(&mut display_info);

    display_info.status("Cache loaded".to_owned());

    let (twete_tx, twete_rx) = chan::sync::<(u8, Vec<u8>)>(0);
    let (coordination_tx, coordination_rx) = chan::sync::<(u8, TwitterConnectionState)>(0);

    for (ref profile_name, ref profile) in &tweeter.profiles {
        connect_twitter_stream(tweeter.app_key.clone(), profile_name.to_string(), profile.creds.clone(), twete_tx.clone(), coordination_tx.clone(), get_id());
    }

    std::thread::spawn(move || {
        for input in stdin().events() {
            ui_tx.send(input);
        }
    });

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

    let termios = Termios::from_fd(0).unwrap();
    let mut new_termios = termios.clone();

    // fix terminal to not echo, thanks
    new_termios.c_lflag &= !(ICANON | ECHO);

    tcsetattr(0, TCSANOW, &new_termios).unwrap();

    match display::paint(&mut tweeter, &mut display_info) {
        Ok(_) => (),
        Err(e) => println!("{}", e)  // TODO: we got here because writing to stdout failed. what to do now?
    };

    do_ui(ui_rx, twete_rx, &twete_tx, coordination_rx, &coordination_tx, &mut tweeter, &mut display_info, &mut queryer);

    tcsetattr(0, TCSANOW, &termios);
}

fn handle_input(event: termion::event::Event, tweeter: &mut tw::TwitterCache, queryer: &mut ::Queryer, display_info: &mut display::DisplayInfo) {
    match event {
        Event::Key(Key::Backspace) => {
            let new_mode = match display_info.get_mode().clone() {
                None => { display_info.input_buf_pop(); None },
                Some(display::DisplayMode::Compose(msg)) => {
                    let mut newstr = msg.clone();
                    newstr.pop();
                    Some(display::DisplayMode::Compose(newstr))
                },
                Some(display::DisplayMode::Reply(twid, msg)) => {
                    let mut newstr = msg.clone();
                    newstr.pop();
                    Some(display::DisplayMode::Reply(twid, newstr))
                }
            };
            display_info.set_mode(new_mode);
        }
        // would Shift('\n') but.. that doesn't exist.
        // would Ctrl('\n') but.. that doesn't work.
        Event::Key(Key::Ctrl('u')) => {
            let new_mode = match display_info.get_mode().clone() {
                None => { display_info.input_buf_drain(); None},
                Some(display::DisplayMode::Compose(msg)) => {
                    Some(display::DisplayMode::Compose("".to_owned()))
                }
                Some(display::DisplayMode::Reply(twid, msg)) => {
                    Some(display::DisplayMode::Reply(twid, "".to_owned()))
                }
            };
            display_info.set_mode(new_mode);
        }
        Event::Key(Key::Ctrl('n')) => {
            let new_mode = match display_info.get_mode().clone() {
                Some(display::DisplayMode::Compose(msg)) => {
                    Some(display::DisplayMode::Compose(format!("{}{}", msg, "\n")))
                }
                mode @ _ => mode
            };
            display_info.set_mode(new_mode);
        }
        // TODO: ctrl+w
        Event::Key(Key::Char(x)) => {
            // Unlike other cases where we care about DisplayMode here,
            // we can't just set the display mode in this function..
            //
            // commands can change display mode, but might not, so just
            // let them do their thing and only explicitly set display
            // mode when we know we ought to
            match display_info.get_mode().clone() {
                None => {
                    if x == '\n' {
                        let line = display_info.input_buf_drain();
                        tweeter.handle_user_input(line.into_bytes(), queryer, display_info);
                    } else {
                        display_info.input_buf_push(x);
                    }
                }
                Some(display::DisplayMode::Compose(msg)) => {
                    if x == '\n' {
                        // TODO: move this somewhere better.
                        ::commands::twete::send_twete(msg, tweeter, queryer, display_info);
                        display_info.set_mode(None)
                    } else {
                        display_info.set_mode(Some(display::DisplayMode::Compose(format!("{}{}", msg, x))))
                    }
                }
                Some(display::DisplayMode::Reply(twid, msg)) => {
                    if x == '\n' {
                        match tweeter.current_profile().map(|profile| profile.to_owned()) {
                            Some(profile) => {
                                // TODO: move this somewhere better.
                                ::commands::twete::send_reply(msg, twid, tweeter, queryer, profile.creds, display_info);
                            },
                            None => {
                                display_info.status("Cannot reply when not logged in".to_owned());
                            }
                        }
                        display_info.set_mode(None)
                    } else {
                        display_info.set_mode(Some(display::DisplayMode::Reply(twid, format!("{}{}", msg, x))))
                    }
                }
            };
        },
        Event::Key(Key::PageUp) => {
            display_info.adjust_infos_seek(Some(1));
        }
        Event::Key(Key::PageDown) => {
            display_info.adjust_infos_seek(Some(-1));
        }
        Event::Key(Key::Home) => {
            display_info.adjust_log_seek(Some(1));
        }
        Event::Key(Key::End) => {
            display_info.adjust_log_seek(Some(-1));
        }
        Event::Key(Key::Esc) => {
            display_info.set_mode(None);
        }
        Event::Key(_) => { }
        Event::Mouse(_) => { }
        Event::Unsupported(_) => {}
    }
}

fn handle_twitter_line(conn_id: u8, line: Vec<u8>, mut tweeter: &mut tw::TwitterCache, mut queryer: &mut ::Queryer, display_info: &mut display::DisplayInfo) {
    match serde_json::from_slice(&line) {
        Ok(json) => {
            tw::handle_message(conn_id, json, &mut tweeter, display_info, &mut queryer);
            if tweeter.needs_save && tweeter.caching_permitted {
                tweeter.store_cache(display_info);
            }
        },
        Err(e) =>
            display_info.status(format!("Error reading twitter line: {:?}", std::str::from_utf8(&line)))
    }
}

fn do_ui(
    ui_rx: chan::Receiver<Result<termion::event::Event, std::io::Error>>,
    twete_rx: chan::Receiver<(u8, Vec<u8>)>,
    twete_tx: &chan::Sender<(u8, Vec<u8>)>,
    coordination_rx: chan::Receiver<(u8, TwitterConnectionState)>,
    coordination_tx: &chan::Sender<(u8, TwitterConnectionState)>,
    mut tweeter: &mut tw::TwitterCache,
    mut display_info: &mut display::DisplayInfo,
    mut queryer: &mut ::Queryer
) {
    loop {
        chan_select! {
            coordination_rx.recv() -> coordination => {
                match coordination {
                    Some((conn_id, coordination)) => {
                        match coordination {
                            TwitterConnectionState::Connecting(profile_name) => {
                                tweeter.connection_map.insert(conn_id, profile_name);
                            },
                            TwitterConnectionState::Connected => {
                                display_info.status(format!("Stream connected for profile \"{}\"", tweeter.connection_map[&conn_id]));
                            },
                            TwitterConnectionState::Closed => {
                                tweeter.connection_map.remove(&conn_id);
                            }
                        }
                    },
                    None => { /* if this stream closes something is terribly wrong... */ panic!("Coordination tx/rx closed!"); }
                }
            },
            twete_rx.recv() -> twete => match twete {
                Some((conn_id, line)) => handle_twitter_line(conn_id, line, tweeter, queryer, display_info),
                None => {
                    display_info.status("Twitter stream hung up...".to_owned());
                    display::paint(tweeter, display_info).unwrap();
                    return; // if the twitter channel died, something real bad happeneed?
                }
            },
            ui_rx.recv() -> user_input => match user_input {
                Some(Ok(event)) => handle_input(event, tweeter, queryer, display_info),
                Some(Err(_)) => (), /* stdin closed? */
                None => return // UI ded
            }
        }

        // one day display_info should be distinct
        match display::paint(tweeter, display_info) {
            Ok(_) => (),
            Err(e) => println!("{}", e)  // TODO: we got here because writing to stdout failed. what to do now?
        };

        match tweeter.state.clone() {
            tw::AppState::ShowHelp => {
                let mut help_lines: Vec<String> = vec![
                    "  Tweets",
                    " ",
                    "Tweets are identified in four (really, three) ways:",
                    "  twitter:1235     - there's no local copy for it, when you look it up I'll have to ask Twitter for it.",
                    // TODO:
                    "  YYYYMMDD:NNNN    - NNNN'th tweet on YYYYMMDD. Numbered as I got them, not by date tweet was made. For example, 20170803:NNNN. NOTE: not currently well supported. Don't even try to use.",
                    "  :NNNN            - NNNN'th tweet since the first I saw.",
                    // TODO:
                    "  NNNN             - NNNN'th tweet of today. Again, numbered by the order I saw it. NOTE: currently this isn't well supported. Use the :ID format above with the same number for now",
                    "    (ex: you want to reply to a tweet `id 1234`, do `rep :1234`, with :, rather than without)",
                    " ",
                    "Tweets can be made immediately by providing the text as part of a command,",
                    "  (like `t hello, world!`)",
                    "or in \"compose mode\" with relevant context shown. If you end up in compose mode and want to get back to a normal prompt, press escape.",
                    " ",
                    "  Commands",
                    " "
                ].into_iter().map(|x| x.to_owned()).collect();
                for command in commands::COMMANDS {
                    help_lines.push(format!("{}{: <width$} {}", command.keyword, command.param_str, command.help_str, width=(35 - command.keyword.len())));
                }
                display_info.recv(display::Infos::Text(help_lines));
                display::paint(tweeter, display_info).unwrap();
                tweeter.state = tw::AppState::View;
            }
            tw::AppState::Reconnect(profile_name) => {
                tweeter.state = tw::AppState::View;
                match tweeter.profiles.get(&profile_name).map(|profile| profile.creds.to_owned()) {
                    Some(user_creds) => {
                        connect_twitter_stream(tweeter.app_key.clone(), profile_name, user_creds, twete_tx.clone(), coordination_tx.clone(), get_id())
                    },
                    None => {
                        display_info.status(format!("No profile named {}", profile_name));
                    }
                }
            },
            tw::AppState::Shutdown => {
                display_info.status("Saving cache...".to_owned());
                display::paint(tweeter, display_info).unwrap();
                tweeter.store_cache(display_info);
                display_info.status("Bye bye!".to_owned());
                display::paint(tweeter, display_info).unwrap();
                return;
            },
            tw::AppState::View | tw::AppState::Compose => { /* nothing special to do */ }
        };
    }
}

fn url_encode(s: &str) -> String {
    fn encode_byte(c: u8) -> String {
        if c == 0x20 {
            "+".to_string()
        } else if (c > 0x40 && c <= 0x40 + 26) ||
           (c > 0x60 && c <= 0x60 + 26) ||
           (c >= 0x30 && c < 0x3a) ||
           c == 0x2d || c == 0x2e || c == 0x5f || c == 0x7e  {
            String::from_utf8(vec![c]).unwrap()
        } else {
            String::from(format!("%{:2x}", c))
        }
    }
    s.as_bytes().iter().map(|c| {
        encode_byte(*c)
    }).collect::<Vec<String>>().join("")
}

//    let (twete_tx, twete_rx) = chan::sync::<Vec<u8>>(0);
#[derive(Debug)]
enum TwitterConnectionState {
    Connecting(String),
    Connected,
    Closed
}
fn connect_twitter_stream(
    app_cred: tw::Credential,
    profile_name: String,
    user_cred: tw::Credential,
    twete_tx: chan::Sender<(u8, Vec<u8>)>,
    coordination_tx: chan::Sender<(u8, TwitterConnectionState)>,
    conn_id: u8
) {
    std::thread::spawn(move || {
        coordination_tx.send((conn_id, TwitterConnectionState::Connecting(profile_name)));
        let mut core = Core::new().unwrap();

        let connector = HttpsConnector::new(1, &core.handle()).unwrap();

        let client = Client::configure()
            .keep_alive(true)
            .connector(connector)
            .build(&core.handle());

        let req = signed_api_get(STREAMURL, &vec![], &app_cred, &user_cred);
        let work = client.request(req).and_then(|res| {
            let status = res.status();
            if status != hyper::StatusCode::Ok {
                println!("Twitter stream connect was abnormal: {}", status);
                println!("result: {:?}", res);
            }
            coordination_tx.send((conn_id, TwitterConnectionState::Connected));
            LineStream::new(res.body()
                .map(|chunk| futures::stream::iter_ok(chunk.into_iter()))
                .flatten())
                .for_each(|s| {
                    if s.len() != 1 {
                        twete_tx.send((conn_id, s));
                    };
                    Ok(())
                })
        });

        let resp = core.run(work);
        match resp {
            Ok(_good) => (),
            Err(e) => println!("Error in setting up: {}", e)
        }
        coordination_tx.send((conn_id, TwitterConnectionState::Closed));
    });
}
