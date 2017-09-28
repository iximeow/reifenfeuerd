extern crate serde_json;

use std::str;
use std::str::FromStr;
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

//Change these values to your real Twitter API credentials
static consumer_key: &str = "T879tHWDzd6LvKWdYVfbJL4Su";
static consumer_secret: &str = "OAXXYYIozAZ4vWSmDziI1EMJCKXPmWPFgLbJpB896iIAMIAdpb";
static token: &str = "629126745-Qt6LPq2kR7w58s7WHzSqcs4CIdiue64kkfYYB7RI";
static token_secret: &str = "3BI3YC4WVbKW5icpHORWpsTYqYIj5oAZFkrgyIAoaoKnK";
static lol_auth_token: &str = "641cdf3a4bbddb72c118b5821e8696aee6300a9a";

static STREAMURL: &str = "https://userstream.twitter.com/1.1/user.json?tweet_mode=extended";
static TWEET_LOOKUP_URL: &str = "https://api.twitter.com/1.1/statuses/show.json?tweet_mode=extended";
static USER_LOOKUP_URL: &str = "https://api.twitter.com/1.1/users/show.json";
static FAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/create.json";
static UNFAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/destroy.json";
static DEL_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/destroy";
static RT_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/retweet";
static CREATE_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/update.json";
static ACCOUNT_SETTINGS_URL: &str = "https://api.twitter.com/1.1/account/settings.json";

header! { (Authorization, "Authorization") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (ContentType, "Content-Type") => [String] }
header! { (Cookie, "cookie") => [String] }

mod tw {
    use std::path::Path;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read};
    extern crate chrono;

    use self::chrono::prelude::*;

    use std::collections::{HashMap, HashSet};
    extern crate serde_json;
    use std::io::Write;

    use std::fs::OpenOptions;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct User {
        pub id: String,
        pub name: String,
        pub handle: String
    }

    impl Default for User {
        fn default() -> User {
            User {
                id: "".to_owned(),
                name: "_default_".to_owned(),
                handle: "_default_".to_owned()
            }
        }
    }

    pub mod events {
        extern crate termion;
        use self::termion::color;

        extern crate serde_json;

        pub struct Deleted {
            user_id: String,
            twete_id: String
        }

        pub struct RT_RT {
            user_id: String,
            twete_id: String
        }

        pub struct Fav_RT {
            user_id: String,
            twete_id: String
        }

        pub struct Fav {
            user_id: String,
            twete_id: String
        }

        pub struct Unfav {
            user_id: String,
            twete_id: String
        }

        pub struct Followed {
            user_id: String
        }

        pub struct Unfollowed {
            user_id: String
        }

        impl Event for Deleted {
            fn render(self: Box<Self>, _tweeter: &::tw::TwitterCache) { }
        }
        impl Event for RT_RT {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("  +rt_rt    : {} (@{})", user.name, user.handle);
                }
                {
                    ::render_twete(&self.twete_id, tweeter);
                }
                println!("");
            }
        }
        impl Event for Fav_RT {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("  +rt_fav   : {} (@{})", user.name, user.handle);
                }
                {
                    ::render_twete(&self.twete_id, tweeter);
                }
                println!("");
            }
        }
        impl Event for Fav {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("{}  +fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset));
                }
                {
                    ::render_twete(&self.twete_id, tweeter);
                }
                println!("");
            }
        }
        impl Event for Unfav {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("{}  -fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset));
                }
                {
                    ::render_twete(&self.twete_id, tweeter);
                }
                println!("");
            }
        }
        impl Event for Followed {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                let user = tweeter.retrieve_user(&self.user_id).unwrap();
                println!("---------------------------------");
                println!("  +fl       : {} (@{})", user.name, user.handle);
                println!("");
            }
        }
        impl Event for Unfollowed {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                let user = tweeter.retrieve_user(&self.user_id).unwrap();
                println!("---------------------------------");
                println!("  -fl       : {} (@{})", user.name, user.handle);
                println!("");
            }
        }

        /*
        impl Event for Blocked {

        }
        */

        pub trait Event {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache);
        }

        impl Event {
            pub fn from_json(structure: serde_json::Map<String, serde_json::Value>) -> Option<Box<Event>> {
                match &structure["event"].as_str().unwrap() {
                    &"follow" => Some(Box::new(Followed {
                        user_id: structure["source"]["id_str"].as_str().unwrap().to_owned()
                    })),
                    &"unfollow" => Some(Box::new(Unfollowed {
                        user_id: structure["source"]["id_str"].as_str().unwrap().to_owned()
                    })),
                    &"favorite" => Some(Box::new(Fav {
                        user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                        twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
                    })),
                    &"unfavorite" => Some(Box::new(Unfav {
                        user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                        twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
                    })),
                    &"favorited_retweet" => Some(Box::new(Fav_RT {
                        user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                        twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
                    })),
                    &"retweeted_retweet" => Some(Box::new(RT_RT {
                        user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                        twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
                    })),
    //                &"blocked" => Blocked { },
    //                &"unblocked" => Unblocked { },
    //                &"quoted_tweet" => ???,
                    e => { println!("unrecognized event: {}", e); None }
                }
            }
        }
    }

    impl User {
        pub fn from_json(json: serde_json::Value) -> Option<User> {
            if let serde_json::Value::Object(json_map) = json {
                if json_map.contains_key("id_str") &&
                   json_map.contains_key("name") &&
                   json_map.contains_key("screen_name") {
                    if let (
                        Some(id_str),
                        Some(name),
                        Some(screen_name)
                    ) = (
                        json_map["id_str"].as_str(),
                        json_map["name"].as_str(),
                        json_map["screen_name"].as_str()
                    ) {
                        return Some(User {
                            id: id_str.to_owned(),
                            name: name.to_owned(),
                            handle: screen_name.to_owned()
                        })
                    }
                }
            }
            None
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Tweet {
        pub id: String,
        pub author_id: String,
        pub text: String,
        pub created_at: String,     // lol
        #[serde(skip_serializing_if="Option::is_none")]
        #[serde(default = "Option::default")]
        pub quoted_tweet_id: Option<String>,
        #[serde(skip_serializing_if="Option::is_none")]
        #[serde(default = "Option::default")]
        pub rt_tweet: Option<String>,
        #[serde(skip)]
        pub internal_id: u64
    }

    impl Tweet {
        pub fn from_api_json(json: serde_json::Value) -> Option<(Tweet, User)> {
            Tweet::from_json(json.clone()).and_then(|tw| {
                json.get("user").and_then(|user_json|
                    User::from_json(user_json.to_owned()).map(|u| (tw, u))
                )
            })
        }
        pub fn from_json(json: serde_json::Value) -> Option<Tweet> {
            if let serde_json::Value::Object(json_map) = json {
                let text = full_twete_text(&json_map);
                let rt_twete = json_map.get("retweeted_status")
                    .and_then(|x| x.get("id_str"))
                    .and_then(|x| x.as_str())
                    .map(|x| x.to_owned());
                if json_map.contains_key("id_str") &&
                   json_map.contains_key("user") &&
                   json_map.contains_key("created_at") {
                    if let (
                        Some(id_str),
                        Some(author_id),
                        Some(created_at)
                    ) = (
                        json_map["id_str"].as_str(),
                        json_map["user"]["id_str"].as_str(),
                        json_map["created_at"].as_str()
                    ) {
                        return Some(Tweet {
                            id: id_str.to_owned(),
                            author_id: author_id.to_owned(),
                            text: text,
                            created_at: created_at.to_owned(),
                            quoted_tweet_id: json_map.get("quoted_status_id_str")
                                .and_then(|x| x.as_str())
                                .map(|x| x.to_owned()),
                            rt_tweet: rt_twete,
                            internal_id: 0
                        })
                    }
                }
            }
            None
        }
    }

    pub fn full_twete_text(twete: &serde_json::map::Map<String, serde_json::Value>) -> String {
        if twete.contains_key("retweeted_status") {
            return full_twete_text(twete["retweeted_status"].as_object().unwrap())
        }
        let mut twete_text: String;
        twete_text = if twete["truncated"].as_bool().unwrap() {
            twete["extended_tweet"]["full_text"].as_str().unwrap().to_string()
        } else {
            twete["text"].as_str().unwrap().to_string()
        };

        let quoted_tweet_id = twete.get("quoted_tweet_id_str").and_then(|x| x.as_str());

        twete_text = twete_text
            .replace("&amp;", "&")
            .replace("&gt;", ">")
            .replace("&lt;", "<");

        for url in twete["entities"]["urls"].as_array().unwrap() {
            let display_url = url["url"].as_str().unwrap();
            let expanded_url = url["expanded_url"].as_str().unwrap();
            if expanded_url.len() < 200 {
                if let Some(twid) = quoted_tweet_id {
                    if expanded_url.ends_with(twid) {
                        twete_text = twete_text.replace(display_url, "");
                        continue;
                    }
                }
                twete_text = twete_text.replace(display_url, expanded_url);
            }
        }

        twete_text
    }

    #[derive(Serialize, Deserialize)]
    pub struct TwitterCache {
        #[serde(skip)]
        pub users: HashMap<String, User>,
        #[serde(skip)]
        pub tweets: HashMap<String, Tweet>,
        following: HashSet<String>,
        following_history: HashMap<String, (String, i64)>, // userid:date??
        pub followers: HashSet<String>,
        lost_followers: HashSet<String>,
        follower_history: HashMap<String, (String, i64)>, // userid:date??
        #[serde(skip)]
        id_to_tweet_id: HashMap<u64, String>,
        #[serde(skip)]
        pub needs_save: bool,
        #[serde(skip)]
        pub caching_permitted: bool,
        #[serde(skip)]
        pub current_user: User
    }

    impl TwitterCache {
        const PROFILE_DIR: &'static str = "cache/";
        const TWEET_CACHE: &'static str = "cache/tweets.json";
        const USERS_CACHE: &'static str = "cache/users.json";
        const PROFILE_CACHE: &'static str = "cache/profile.json"; // this should involve MY user id..

        fn new() -> TwitterCache {
            TwitterCache {
                users: HashMap::new(),
                tweets: HashMap::new(),
                following: HashSet::new(),
                following_history: HashMap::new(),
                followers: HashSet::new(),
                lost_followers: HashSet::new(),
                follower_history: HashMap::new(),
                id_to_tweet_id: HashMap::new(),
                needs_save: false,
                caching_permitted: true,
                current_user: User::default()
            }
        }
        fn new_without_caching() -> TwitterCache {
            let mut cache = TwitterCache::new();
            cache.caching_permitted = false;
            cache
        }
        fn cache_user(&mut self, user: User) {
            if !self.users.contains_key(&user.id) {
                let mut file =
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(TwitterCache::USERS_CACHE)
                        .unwrap();
                writeln!(file, "{}", serde_json::to_string(&user).unwrap()).unwrap();
                self.users.insert(user.id.to_owned(), user);
            }
        }

        fn cache_tweet(&mut self, tweet: Tweet) {
            if !self.tweets.contains_key(&tweet.id) {

                let mut file =
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(TwitterCache::TWEET_CACHE)
                        .unwrap();
                writeln!(file, "{}", serde_json::to_string(&tweet).unwrap()).unwrap();
                self.number_and_insert_tweet(tweet);
            }
        }
        pub fn store_cache(&self) {
            if Path::new(TwitterCache::PROFILE_DIR).is_dir() {
                let profile = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(false)
                    .open(TwitterCache::PROFILE_CACHE)
                    .unwrap();
                serde_json::to_writer(profile, self).unwrap();
            } else {
                println!("No cache dir exists...");
            }
            // store cache
        }
        fn number_and_insert_tweet(&mut self, mut tw: Tweet) {
            if !self.tweets.contains_key(&tw.id.to_owned()) {
                if tw.internal_id == 0 {
                    tw.internal_id = (self.tweets.len() as u64) + 1;
                    self.id_to_tweet_id.insert(tw.internal_id, tw.id.to_owned());
                    self.tweets.insert(tw.id.to_owned(), tw);
                }
            }
        }
        pub fn load_cache() -> TwitterCache {
            if Path::new(TwitterCache::PROFILE_CACHE).is_file() {
                let mut buf = vec![];
                let mut profile = File::open(TwitterCache::PROFILE_CACHE).unwrap();
                match profile.read_to_end(&mut buf) {
                    Ok(_sz) => {
                        match serde_json::from_slice(&buf) {
                            Ok(result) => {
                                let mut cache: TwitterCache = result;
                                cache.tweets = HashMap::new();
                                for line in BufReader::new(File::open(TwitterCache::TWEET_CACHE).unwrap()).lines() {
                                    let t: Tweet = serde_json::from_str(&line.unwrap()).unwrap();
                                    cache.number_and_insert_tweet(t);
                                }
                                for line in BufReader::new(File::open(TwitterCache::USERS_CACHE).unwrap()).lines() {
                                    let u: User = serde_json::from_str(&line.unwrap()).unwrap();
                                    cache.users.insert(u.id.to_owned(), u);
                                }
                                cache.caching_permitted = true;
                                cache.needs_save = false;
                                cache
                            }
                            Err(e) => {
                                // TODO! should be able to un-frick profile after startup.
                                println!("Error reading profile, profile caching disabled... {}", e);
                                TwitterCache::new_without_caching()
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading cached profile: {}. Profile caching disabled.", e);
                        TwitterCache::new_without_caching()
                    }
                }
            } else {
                println!("Hello! First time setup?");
                TwitterCache::new()
            }
        }
        pub fn cache_api_tweet(&mut self, json: serde_json::Value) {
            if let Some((rt, rt_user)) = json.get("retweeted_status").and_then(|x| Tweet::from_api_json(x.to_owned())) {
                self.cache_user(rt_user);
                self.cache_tweet(rt);
            }

            if let Some((qt, qt_user)) = json.get("quoted_status").and_then(|x| Tweet::from_api_json(x.to_owned())) {
                self.cache_user(qt_user);
                self.cache_tweet(qt);
            }

            if let Some((twete, user)) = Tweet::from_api_json(json) {
                self.cache_user(user);
                self.cache_tweet(twete);
            }
        }
        pub fn cache_api_user(&mut self, json: serde_json::Value) {
            if let Some(user) = User::from_json(json) {
                self.cache_user(user);
            }
        }
        pub fn cache_api_event(&mut self, json: serde_json::Map<String, serde_json::Value>, mut queryer: &mut ::Queryer) {
            /* don't really care to hold on to who fav, unfav, ... when, just pick targets out. */
            match json.get("event").and_then(|x| x.as_str()) {
                Some("favorite") => {
                    self.cache_api_tweet(json["target_object"].clone());
                    self.cache_api_user(json["source"].clone());
                    self.cache_api_user(json["target"].clone());
                },
                Some("unfavorite") => {
                    self.cache_api_tweet(json["target_object"].clone());
                    self.cache_api_user(json["source"].clone());
                    self.cache_api_user(json["target"].clone());
                },
                Some("retweeted_retweet") => ()/* cache rt */,
                Some("favorited_retweet") => ()/* cache rt */,
                Some("delete") => {
                    let user_id = json["delete"]["status"]["user_id_str"].as_str().unwrap().to_string();
                    self.fetch_user(&user_id, &mut queryer);
                },
                Some("follow") => {
                    let follower = json["source"]["id_str"].as_str().unwrap().to_string();
                    let followed = json["target"]["id_str"].as_str().unwrap().to_string();
                    self.cache_api_user(json["target"].clone());
                    self.cache_api_user(json["source"].clone());
                    if follower == "iximeow" {
                        // self.add_follow(
                    } else {
                        self.add_follower(&follower);
                    }
                },
                Some("unfollow") => {
                    let follower = json["source"]["id_str"].as_str().unwrap().to_string();
                    let followed = json["target"]["id_str"].as_str().unwrap().to_string();
                    self.cache_api_user(json["target"].clone());
                    self.cache_api_user(json["source"].clone());
                    if follower == "iximeow" {
                        // self.add_follow(
                    } else {
                        self.remove_follower(&follower);
                    }
                },
                Some(_) => () /* an uninteresting event */,
                None => () // not really an event? should we log something?
                /* nothing else to care about now, i think? */
            }
        }
        pub fn tweet_by_innerid(&self, inner_id: u64) -> Option<&Tweet> {
            let id = &self.id_to_tweet_id[&inner_id];
            self.retrieve_tweet(id)
        }
        pub fn retrieve_tweet(&self, tweet_id: &String) -> Option<&Tweet> {
            self.tweets.get(tweet_id)
        }
        pub fn retrieve_user(&self, user_id: &String) -> Option<&User> {
            self.users.get(user_id)
        }
        pub fn fetch_tweet(&mut self, tweet_id: &String, mut queryer: &mut ::Queryer) -> Option<&Tweet> {
            if !self.tweets.contains_key(tweet_id) {
                match self.look_up_tweet(tweet_id, &mut queryer) {
                    Some(json) => self.cache_api_tweet(json),
                    None => println!("Unable to retrieve tweet {}", tweet_id)
                };
            }
            self.tweets.get(tweet_id)
        }
        pub fn fetch_user(&mut self, user_id: &String, mut queryer: &mut ::Queryer) -> Option<&User> {
            if !self.users.contains_key(user_id) {
                let maybe_parsed = self.look_up_user(user_id, &mut queryer).and_then(|x| User::from_json(x));
                match maybe_parsed {
                    Some(tw) => self.cache_user(tw),
                    None => println!("Unable to retrieve user {}", user_id)
                };
            }
            self.users.get(user_id)
        }
        pub fn set_following(&mut self, user_ids: Vec<String>) {
            let uid_set = user_ids.into_iter().collect::<HashSet<String>>();

            let new_uids = &uid_set - &self.following;
            for user in &new_uids {
                println!("New following! {}", user);
                self.add_following(user);
            }

            let lost_uids = &self.following - &uid_set;
            for user in &lost_uids {
                println!("Bye, friend! {}", user);
                self.remove_following(user);
            }
        }
        pub fn set_followers(&mut self, user_ids: Vec<String>) {
            let uid_set = user_ids.into_iter().collect::<HashSet<String>>();

            let new_uids = &uid_set - &self.followers;
            for user in &new_uids {
                println!("New follower! {}", user);
                self.add_follower(user);
            }

            let lost_uids = &self.followers - &uid_set;
            for user in &lost_uids {
                println!("Bye, friend! {}", user);
                self.remove_follower(user);
            }
        }
        pub fn add_following(&mut self, user_id: &String) {
            self.needs_save = true;
            self.following.insert(user_id.to_owned());
            self.following_history.insert(user_id.to_owned(), ("following".to_string(), Utc::now().timestamp()));
        }
        pub fn remove_following(&mut self, user_id: &String) {
            self.needs_save = true;
            self.following.remove(user_id);
            self.following_history.insert(user_id.to_owned(), ("unfollowing".to_string(), Utc::now().timestamp()));
        }
        pub fn add_follower(&mut self, user_id: &String) {
            self.needs_save = true;
            self.followers.insert(user_id.to_owned());
            self.lost_followers.remove(user_id);
            self.follower_history.insert(user_id.to_owned(), ("follow".to_string(), Utc::now().timestamp()));
        }
        pub fn remove_follower(&mut self, user_id: &String) {
            self.needs_save = true;
            self.followers.remove(user_id);
            self.lost_followers.insert(user_id.to_owned());
            self.follower_history.insert(user_id.to_owned(), ("unfollow".to_string(), Utc::now().timestamp()));
        }

        fn look_up_user(&mut self, id: &str, queryer: &mut ::Queryer) -> Option<serde_json::Value> {
            let url = &format!("{}?user_id={}", ::USER_LOOKUP_URL, id);
            queryer.do_api_get(url)
        }

        fn look_up_tweet(&mut self, id: &str, queryer: &mut ::Queryer) -> Option<serde_json::Value> {
            let url = &format!("{}?id={}", ::TWEET_LOOKUP_URL, id);
            queryer.do_api_get(url)
        }

        pub fn get_settings(&self, queryer: &mut ::Queryer) -> Option<serde_json::Value> {
            queryer.do_api_get(::ACCOUNT_SETTINGS_URL)
        }
    }
}

pub struct Queryer {
    client: hyper::client::Client<HttpsConnector<hyper::client::HttpConnector>>,
    core: Core
}

impl Queryer {
    fn do_api_get(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_api_get(url))
    }
    fn do_api_post(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_api_post(url))
    }
    /*
    fn do_web_req(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_web_get(url))
    }*/
    // TODO: make this return the status as well!
    fn issue_request(&mut self, req: hyper::client::Request) -> Option<serde_json::Value> {
        let lookup = self.client.request(req);

        let resp: hyper::Response = self.core.run(lookup).unwrap();
        let status = resp.status().clone();

        let chunks: Vec<hyper::Chunk> = self.core.run(resp.body().collect()).unwrap();

        let resp_body: Vec<u8> = chunks.into_iter().flat_map(|chunk| chunk.into_iter()).collect();

        match serde_json::from_slice(&resp_body) {
            Ok(value) => {
                if status != hyper::StatusCode::Ok {
                    println!("!! Requests returned status: {}", status);
                    println!("{}", value);
                    None
                } else {
                    Some(value)
                }
            }
            Err(e) => {
                if status != hyper::StatusCode::Ok {
                    println!("!! Requests returned status: {}", status);
                }
                println!("error deserializing json: {}", e);
                None
            }
        }
    }
}

extern crate termion;

use termion::color;

fn color_for(handle: &String) -> termion::color::Fg<&color::Color> {
    let color_map: Vec<&color::Color> = vec![
        &color::Blue,
        &color::Cyan,
        &color::Green,
        &color::LightBlue,
        &color::LightCyan,
        &color::LightGreen,
        &color::LightMagenta,
        &color::LightYellow,
        &color::Magenta,
        &color::Yellow
    ];

    let mut quot_hash_quot = std::num::Wrapping(0);
    for b in handle.as_bytes().iter() {
        quot_hash_quot = quot_hash_quot + std::num::Wrapping(*b);
    }
    color::Fg(color_map[quot_hash_quot.0 as usize % color_map.len()])
}


fn render_twete(twete_id: &String, tweeter: &tw::TwitterCache) {
    let id_color = color::Fg(color::Rgb(180, 80, 40));
    let twete = tweeter.retrieve_tweet(twete_id).unwrap();
    // if we got the tweet, the API gave us the user too
    let user = tweeter.retrieve_user(&twete.author_id).unwrap();
    match twete.rt_tweet {
        Some(ref rt_id) => {
            // same for a retweet
            let rt = tweeter.retrieve_tweet(rt_id).unwrap();
            // and its author
            let rt_author = tweeter.retrieve_user(&rt.author_id).unwrap();
            println!("{}  id:{} (rt_id:{}){}",
                id_color, rt.internal_id, twete.internal_id, color::Fg(color::Reset)
            );
            println!("  {}{}{} ({}@{}{}) via {}{}{} ({}@{}{}) RT:",
                color_for(&rt_author.handle), rt_author.name, color::Fg(color::Reset),
                color_for(&rt_author.handle), rt_author.handle, color::Fg(color::Reset),
                color_for(&user.handle), user.name, color::Fg(color::Reset),
                color_for(&user.handle), user.handle, color::Fg(color::Reset)
            );
        }
        None => {
            println!("{}  id:{}{}",
                id_color, twete.internal_id, color::Fg(color::Reset)
            );
            println!("  {}{}{} ({}@{}{})",
                color_for(&user.handle), user.name, color::Fg(color::Reset),
                color_for(&user.handle), user.handle, color::Fg(color::Reset)
            );
        }
    }

    println!("      {}", twete.text.replace("\r", "\\r").split("\n").collect::<Vec<&str>>().join("\n      "));

    if let Some(ref qt_id) = twete.quoted_tweet_id {
        if let Some(ref qt) = tweeter.retrieve_tweet(qt_id) {
            let qt_author = tweeter.retrieve_user(&qt.author_id).unwrap();
            println!("{}    id:{}{}",
                id_color, qt.internal_id, color::Fg(color::Reset)
            );
            println!(
                "    {}{}{} ({}@{}{})",
                color_for(&qt_author.handle), qt_author.name, color::Fg(color::Reset),
                color_for(&qt_author.handle), qt_author.handle, color::Fg(color::Reset)
            );
            println!(
                "        {}",
                qt.text.replace("\r", "\\r").split("\n").collect::<Vec<&str>>().join("\n        ")
            );
        } else {
            println!("    << don't have quoted tweet! >>");
        }
    }
}

fn render_twitter_event(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut tw::TwitterCache,
    mut queryer: &mut Queryer) {
    if structure.contains_key("event") {
        tweeter.cache_api_event(structure.clone(), &mut queryer);
        if let Some(event) = tw::events::Event::from_json(structure) {
            event.render(&tweeter);
        };
    } else if structure.contains_key("friends") {
//        println!("welcome: {:?}", structure);
        let user_id_nums = structure["friends"].as_array().unwrap();
        let user_id_strs = user_id_nums.into_iter().map(|x| x.as_u64().unwrap().to_string());
        tweeter.set_following(user_id_strs.collect());
        let settings = tweeter.get_settings(queryer).unwrap();
        let maybe_my_name = settings["screen_name"].as_str();
        if let Some(my_name) = maybe_my_name {
            tweeter.current_user = tw::User {
                id: "".to_string(),
                handle: my_name.to_owned(),
                name: my_name.to_owned()
            };
            println!("You are {}", tweeter.current_user.handle);
        } else {
            println!("Unable to make API call to figure out who you are...");
        }
    } else if structure.contains_key("delete") {
        let deleted_user_id = structure["delete"]["status"]["user_id_str"].as_str().unwrap().to_string();
        let deleted_tweet_id = structure["delete"]["status"]["id_str"].as_str().unwrap().to_string();
        if let Some(handle) = tweeter.retrieve_user(&deleted_user_id).map(|x| &x.handle) {
            if let Some(_tweet) = tweeter.retrieve_tweet(&deleted_tweet_id) {
                println!("-------------DELETED------------------");
                render_twete(&deleted_tweet_id, tweeter);
                println!("-------------DELETED------------------");
            } else {
                println!("dunno what, but do know who: {} - {}", deleted_user_id, handle);
            }
        } else {
            println!("delete...");
            println!("dunno who...");
        }
    } else if structure.contains_key("user") && structure.contains_key("id") {
        let twete_id = structure["id_str"].as_str().unwrap().to_string();
        tweeter.cache_api_tweet(serde_json::Value::Object(structure));
        render_twete(&twete_id, tweeter);
    } else if structure.contains_key("direct_message") {
        // show DM
        println!("{}", structure["direct_message"]["text"].as_str().unwrap());
        println!("Unknown struture {:?}", structure);
    }
    println!("");
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

//    println!("Request built: {:?}", req);
    req
}

fn display_event(
    twete: serde_json::Value,
    tweeter: &mut tw::TwitterCache,
    queryer: &mut Queryer
) {
    match twete {
        serde_json::Value::Object(objmap) => render_twitter_event(objmap, tweeter, queryer),
        _ => ()
    };
}

fn main() {

    //Track words
//    let url = "https://stream.twitter.com/1.1/statuses/filter.json";
//    let url = "https://stream.twitter.com/1.1/statuses/sample.json";

    println!("starting!");

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

    println!("Bye bye");
}

fn do_ui(ui_rx_orig: chan::Receiver<Vec<u8>>, twete_rx: chan::Receiver<Vec<u8>>, mut tweeter: &mut tw::TwitterCache, mut queryer: &mut ::Queryer) -> Option<(chan::Receiver<Vec<u8>>, chan::Receiver<Vec<u8>>)> {
    loop {
        let ui_rx_a = &ui_rx_orig;
        let ui_rx_b = &ui_rx_orig;
        chan_select! {
            twete_rx.recv() -> twete => match twete {
                Some(line) => {
                    let jsonstr = std::str::from_utf8(&line).unwrap().trim();
//                    println!("{}", jsonstr);
                    /* TODO: replace from_str with from_slice */
                    let json: serde_json::Value = serde_json::from_str(&jsonstr).unwrap();
                    display_event(json, &mut tweeter, &mut queryer);
                    if tweeter.needs_save && tweeter.caching_permitted {
                        tweeter.store_cache();
                    }
                }
                None => {
                    println!("Twitter stream hung up...");
                    chan_select! {
                        ui_rx_b.recv() -> input => match input {
                            Some(line) => {
                                if line == "reconnect\n".as_bytes() {
                                    return Some((ui_rx_orig.clone(), connect_twitter_stream()));
                                } else {
                                    handle_user_input(line, &mut tweeter, &mut queryer);
                                }
                            }
                            None => std::process::exit(0)
                        }
                    }
                }
            },
            ui_rx_a.recv() -> user_input => match user_input {
                Some(line) => {
                    handle_user_input(line, &mut tweeter, &mut queryer);
                },
                None => println!("UI thread hung up...")
            }
        }
    }
}

fn url_encode(s: &str) -> String {
    s
        .replace(" ", "+")
        .replace("%", "%25")
        .replace("\\n", "%0a")
        .replace("\\r", "%0d")
        .replace("!", "%21")
        .replace("#", "%23")
        .replace("&", "%26")
        .replace("'", "%27")
        .replace("(", "%28")
        .replace(")", "%29")
        .replace("*", "%2a")
//                            .replace("+", "%2b")
        .replace(",", "%2c")
        .replace("-", "%2d")
        .replace(".", "%2e")
        .replace("/", "%2f")
        .replace(":", "%3a")
        .replace(">", "%3e")
        .replace("<", "%3c")
        .replace("?", "%3f")
        .replace("@", "%40")
        .replace("\\", "%5c")
}

struct Command {
    keyword: &'static str,
    params: u8,
    exec: fn(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer)
}

static SHOW_CACHE: Command = Command {
    keyword: "show_cache",
    params: 0,
    exec: |_line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer| {
        println!("----* USERS *----");
        for (uid, user) in &tweeter.users {
            println!("User: {} -> {:?}", uid, user);
        }
        println!("----* TWEETS *----");
        for (tid, tweet) in &tweeter.tweets {
            println!("Tweet: {} -> {:?}", tid, tweet);
        }
        println!("----* FOLLOWERS *----");
        for uid in &tweeter.followers.clone() {
            let user_res = tweeter.fetch_user(uid, &mut queryer);
            match user_res {
                Some(user) => {
                    println!("Follower: {} - {:?}", uid, user);
                }
                None => { println!("  ..."); }
            }
        }
    }
};

static QUIT: Command = Command {
    keyword: "q",
    params: 0,
    exec: |_line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer| {
        println!("Bye bye!");
        tweeter.store_cache();
        std::process::exit(0);
    }
};

static LOOK_UP_USER: Command = Command {
    keyword: "look_up_user",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer| {
        if let Some(user) = tweeter.fetch_user(&line, &mut queryer) {
            println!("{:?}", user);
        } else {
//            println!("Couldn't retrieve {}", userid);
        }
    }
};

static LOOK_UP_TWEET: Command = Command {
    keyword: "look_up_tweet",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer| {
        if let Some(tweet) = tweeter.fetch_tweet(&line, &mut queryer) {
            println!("{:?}", tweet);
        } else {
//            println!("Couldn't retrieve {}", tweetid);
        }
    }
};

static VIEW: Command = Command {
    keyword: "view",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer| {
        // TODO handle this unwrap
        let inner_twid = u64::from_str(&line).unwrap();
        let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
        render_twete(&twete.id, tweeter);
        println!("link: https://twitter.com/i/web/status/{}", twete.id);
    }
};

static UNFAV: Command = Command {
    keyword: "unfav",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        // TODO handle this unwrap
        let inner_twid = u64::from_str(&line).unwrap();
        let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
        queryer.do_api_post(&format!("{}?id={}", UNFAV_TWEET_URL, twete.id));
    }
};

static FAV: Command = Command {
    keyword: "fav",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        // TODO handle this unwrap
        let inner_twid = u64::from_str(&line).unwrap();
        let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
        queryer.do_api_post(&format!("{}?id={}", FAV_TWEET_URL, twete.id));
    }
};

static DEL: Command = Command {
    keyword: "del",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        let inner_twid = u64::from_str(&line).unwrap();
        let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
        queryer.do_api_post(&format!("{}/{}.json", DEL_TWEET_URL, twete.id));
    }
};

static TWETE: Command = Command {
    keyword: "t",
    params: 1,
    exec: |line: String, _tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        let text = line.trim();
        let substituted = url_encode(text);
        println!("msg len: {}", text.len());
        println!("excessively long? {}", text.len() > 140);
        if text.len() > 140 {
            queryer.do_api_post(&format!("{}?status={}", CREATE_TWEET_URL, substituted));
        } else {
            queryer.do_api_post(&format!("{}?status={}&weighted_character_count=true", CREATE_TWEET_URL, substituted));
        }
//        println!("{}", &format!("{}?status={}", CREATE_TWEET_URL, substituted));
    }
};

static THREAD: Command = Command {
    keyword: "thread",
    params: 2,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        let mut text: String = line.trim().to_string();
        if let Some(id_end_idx) = text.find(" ") {
            let reply_bare = text.split_off(id_end_idx + 1);
            let reply = reply_bare.trim();
            let id_str = text.trim();
            if reply.len() > 0 {
                if let Some(inner_twid) = u64::from_str(&id_str).ok() {
                    if let Some(twete) = tweeter.tweet_by_innerid(inner_twid) {
                        let handle = &tweeter.retrieve_user(&twete.author_id).unwrap().handle;
                        // TODO: definitely breaks if you change your handle right now
                        if handle == &tweeter.current_user.handle {
                            let substituted = url_encode(reply);
                            queryer.do_api_post(&format!("{}?status={}&in_reply_to_status_id={}", CREATE_TWEET_URL, substituted, twete.id));
                        } else {
                            println!("you can only thread your own tweets");
                            // ask if it should .@ instead?
                        }
                        let substituted = url_encode(reply);
                        queryer.do_api_post(&format!("{}?status={}&in_reply_to_status_id={}", CREATE_TWEET_URL, substituted, twete.id));
                    }
                }
            } else {
                println!("thread <id> your sik reply");
            }
        } else {
            println!("thread <id> your sik reply");
        }
    }
};

static REP: Command = Command {
    keyword: "rep",
    params: 2,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        let mut text: String = line.trim().to_string();
        if let Some(id_end_idx) = text.find(" ") {
            let reply_bare = text.split_off(id_end_idx + 1);
            let reply = reply_bare.trim();
            let id_str = text.trim();
            if reply.len() > 0 {
                if let Some(inner_twid) = u64::from_str(&id_str).ok() {
                    if let Some(twete) = tweeter.tweet_by_innerid(inner_twid) {
                        let substituted = url_encode(reply);
                        queryer.do_api_post(&format!("{}?status={}&in_reply_to_status_id={}", CREATE_TWEET_URL, substituted, twete.id));
                    }
                }
            } else {
                println!("rep <id> your sik reply");
            }
        } else {
            println!("rep <id> your sik reply");
        }
    }
};

static QUOTE: Command = Command {
    keyword: "qt",
    params: 2,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        let mut text: String = line.trim().to_string();
        if let Some(id_end_idx) = text.find(" ") {
            let reply_bare = text.split_off(id_end_idx + 1);
            let reply = reply_bare.trim();
            let id_str = text.trim();
            if reply.len() > 0 {
                if let Some(inner_twid) = u64::from_str(&id_str).ok() {
                    if let Some(twete) = tweeter.tweet_by_innerid(inner_twid) {
                        let substituted = url_encode(reply);
                        let attachment_url = url_encode(
                            &format!(
                                "https://www.twitter.com/{}/status/{}",
                                tweeter.retrieve_user(&twete.author_id).unwrap().handle,
                                twete.id
                            )
                        );
                        println!("{}", substituted);
                        queryer.do_api_post(
                            &format!("{}?status={}&attachment_url={}",
                                     CREATE_TWEET_URL,
                                     substituted,
                                     attachment_url
                            )
                        );
                    }
                }
            } else {
                println!("rep <id> your sik reply");
            }
        } else {
            println!("rep <id> your sik reply");
        }
    }
};

static RETWETE: Command = Command {
    keyword: "rt",
    params: 1,
    exec: |line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer| {
        let inner_twid = u64::from_str(&line).unwrap();
        let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
        queryer.do_api_post(&format!("{}/{}.json", RT_TWEET_URL, twete.id));
    }
};

fn parse_word_command<'a, 'b>(line: &'b str, commands: Vec<&'a Command>) -> Option<(&'b str, &'a Command)> {
    for cmd in commands.into_iter() {
        if cmd.params == 0 {
            if line == cmd.keyword {
                return Some(("", &cmd));
            }
        } else if line.starts_with(cmd.keyword) {
            // let inner_twid = u64::from_str(&linestr.split(" ").collect::<Vec<&str>>()[1]).unwrap();
            return Some((line.get((cmd.keyword.len() + 1)..).unwrap().trim(), &cmd));
        }
    }
    return None
}

fn handle_user_input(line: Vec<u8>, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer) {
    let commands = vec![
        &SHOW_CACHE,
        &QUIT,
        &LOOK_UP_USER,
        &LOOK_UP_TWEET,
        &VIEW,
        &UNFAV,
        &FAV,
        &DEL,
        &TWETE,
        &QUOTE,
        &RETWETE,
        &REP,
        &THREAD
    ];
    let command_bare = String::from_utf8(line).unwrap();
    let command = command_bare.trim();
    if let Some((line, cmd)) = parse_word_command(&command, commands) {
        (cmd.exec)(line.to_owned(), tweeter, &mut queryer);
    } else {
        println!("I don't know what {} means", command);
    }
    println!(""); // temporaryish because there's no visual distinction between output atm
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

    //    println!("{}", do_web_req("https://caps.twitter.com/v2/capi/passthrough/1?twitter:string:card_uri=card://887655800482787328&twitter:long:original_tweet_id=887655800981925888&twitter:string:response_card_name=poll3choice_text_only&twitter:string:cards_platform=Web-12", &client, &mut core).unwrap());
    //    println!("{}", look_up_tweet("887655800981925888", &client, &mut core).unwrap());

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

