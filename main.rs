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

//Change these values to your real Twitter API credentials
static consumer_key: &str = "T879tHWDzd6LvKWdYVfbJL4Su";
static consumer_secret: &str = "OAXXYYIozAZ4vWSmDziI1EMJCKXPmWPFgLbJpB896iIAMIAdpb";
static token: &str = "629126745-Qt6LPq2kR7w58s7WHzSqcs4CIdiue64kkfYYB7RI";
static token_secret: &str = "3BI3YC4WVbKW5icpHORWpsTYqYIj5oAZFkrgyIAoaoKnK";
static lol_auth_token: &str = "641cdf3a4bbddb72c118b5821e8696aee6300a9a";

static streamurl: &str = "https://userstream.twitter.com/1.1/user.json?tweet_mode=extended";
static tweet_lookup_url: &str = "https://api.twitter.com/1.1/statuses/show.json?tweet_mode=extended";
static user_lookup_url: &str = "https://api.twitter.com/1.1/users/show.json";

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

    extern crate hyper;
    extern crate hyper_tls;
    use std::collections::{HashMap, HashSet};
    use tokio_core::reactor::Core;
    use hyper_tls::HttpsConnector;
    extern crate serde_json;
    extern crate serde;
    extern crate serde_derive;
    use std::io::Write;

    use std::fs::OpenOptions;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct User {
        pub id: String,
        pub name: String,
        pub handle: String
    }

    pub mod events {
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
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) { }
        }
        impl Event for RT_RT {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("  +rt_rt    : {} (@{})", user.name, user.handle);
                }
                {
                    let target = tweeter.retrieve_tweet(&self.twete_id).unwrap();
                    ::render_twete(target, tweeter);
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
                    let target = tweeter.retrieve_tweet(&self.twete_id).unwrap();
                    ::render_twete(target, tweeter);
                }
                println!("");
            }
        }
        impl Event for Fav {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("  +fav      : {} (@{})", user.name, user.handle);
                }
                {
                    let target = tweeter.retrieve_tweet(&self.twete_id).unwrap();
                    ::render_twete(target, tweeter);
                }
                println!("");
            }
        }
        impl Event for Unfav {
            fn render(self: Box<Self>, tweeter: &::tw::TwitterCache) {
                println!("---------------------------------");
                {
                    let user = tweeter.retrieve_user(&self.user_id).unwrap();
                    println!("  +fav      : {} (@{})", user.name, user.handle);
                }
                {
                    let target = tweeter.retrieve_tweet(&self.twete_id).unwrap();
                    ::render_twete(target, tweeter);
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
        pub rt_tweet: Option<String>
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
                            rt_tweet: rt_twete
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
        pub needs_save: bool,
        #[serde(skip)]
        pub caching_permitted: bool
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
                needs_save: false,
                caching_permitted: true
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
                writeln!(file, "{}", serde_json::to_string(&user).unwrap());
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
                writeln!(file, "{}", serde_json::to_string(&tweet).unwrap());
                self.tweets.insert(tweet.id.to_owned(), tweet);
            }
        }
        pub fn store_cache(&self) {
            if Path::new(TwitterCache::PROFILE_DIR).is_dir() {
                let mut profile = OpenOptions::new()
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
        pub fn load_cache() -> TwitterCache {
            if Path::new(TwitterCache::PROFILE_CACHE).is_file() {
                let mut buf = vec![];
                let mut profile = File::open(TwitterCache::PROFILE_CACHE).unwrap();
                match profile.read_to_end(&mut buf) {
                    Ok(sz) => {
                        match serde_json::from_slice(&buf) {
                            Ok(result) => {
                                let mut cache: TwitterCache = result;
                                cache.tweets = HashMap::new();
                                for line in BufReader::new(File::open(TwitterCache::TWEET_CACHE).unwrap()).lines() {
                                    let t: Tweet = serde_json::from_str(&line.unwrap()).unwrap();
                                    cache.tweets.insert(t.id.to_owned(), t);
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
                    if follower == "iximeow" {
                        // self.add_follow(
                    } else {
                        self.add_follower(&follower);
                    }
                },
                Some("unfollow") => {
                    let follower = json["source"]["id_str"].as_str().unwrap().to_string();
                    let followed = json["target"]["id_str"].as_str().unwrap().to_string();
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
            let url = &format!("{}?user_id={}", ::user_lookup_url, id);
            queryer.do_api_req(url)
        }

        fn look_up_tweet(&mut self, id: &str, queryer: &mut ::Queryer) -> Option<serde_json::Value> {
            let url = &format!("{}?id={}", ::tweet_lookup_url, id);
            queryer.do_api_req(url)
        }
    }
}

pub struct Queryer {
    client: hyper::client::Client<HttpsConnector<hyper::client::HttpConnector>>,
    core: Core
}

impl Queryer {
    fn do_api_req(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_api_get(url))
    }
    fn do_web_req(&mut self, url: &str) -> Option<serde_json::Value> {
        self.issue_request(signed_web_get(url))
    }
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
                println!("error deserializing json: {}", e);
                None
            }
        }
    }
}


fn render_twete(twete: &tw::Tweet, tweeter: &tw::TwitterCache) {
    // if we got the tweet, the API gave us the user too
    let user = tweeter.retrieve_user(&twete.author_id).unwrap();
    match twete.rt_tweet {
        Some(ref rt_id) => {
            // same for a retweet
            let rt = tweeter.retrieve_tweet(rt_id).unwrap();
            // and its author
            let rt_author = tweeter.retrieve_user(&rt.author_id).unwrap();
            println!("{} (@{}) via {} (@{}) RT:                           https://twitter.com/i/web/status/{}", rt_author.name, rt_author.handle, user.name, user.handle, rt.id);
        }
        None => {
            println!("{} (@{})                                            https://twitter.com/i/web/status/{}", user.name, user.handle, twete.id);
        }
    }

    println!("{}", twete.text);

    if let Some(ref qt_id) = twete.quoted_tweet_id {
        let qt = tweeter.retrieve_tweet(qt_id).unwrap();
        let qt_author = tweeter.retrieve_user(&qt.author_id).unwrap();
        println!(
            "  {} (@{})                                             https://twitter.com/i/web/status/{}\n    {}",
            qt_author.name,
            qt_author.handle,
            qt.id,
            qt.text.split("\n").collect::<Vec<&str>>().join("\n    ")
        );
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
        let user_id_nums = structure["friends"].as_array().unwrap();
        let user_id_strs = user_id_nums.into_iter().map(|x| x.as_u64().unwrap().to_string());
        tweeter.set_following(user_id_strs.collect());
    } else if structure.contains_key("delete") {
        println!("delete...");
        let deleted_user_id = structure["delete"]["status"]["user_id_str"].as_str().unwrap().to_string();
        if let Some(handle) = tweeter.retrieve_user(&deleted_user_id).map(|x| &x.handle) {
            println!("who? {} - {}", deleted_user_id, handle);
        } else {
            println!("dunno who...");
        }
    } else if structure.contains_key("user") && structure.contains_key("id") {
        let twete_id = structure["id_str"].as_str().unwrap().to_string();
        tweeter.cache_api_tweet(serde_json::Value::Object(structure));
        render_twete(tweeter.retrieve_tweet(&twete_id).unwrap(), tweeter);
    }
    println!("");
}

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
        headers.set(Accept("*/*".to_owned()));
        headers.set(ContentType("application/x-www-form-urlencoded".to_owned()));
    };

    req
}

fn signed_api_get(url: &str) -> hyper::client::Request {
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
        println!("{}", header.to_owned());
        let mut headers = req.headers_mut();
        headers.set(Authorization(header.to_owned()));
        headers.set(Accept("*/*".to_owned()));
        headers.set(ContentType("application/x-www-form-urlencoded".to_owned()));
    };

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

    let (ui_tx, ui_rx) = chan::sync::<Vec<u8>>(0);

    let twete_rx = connect_twitter_stream();

    std::thread::spawn(move || {
        use std::io::Read;
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line);
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
    let secondaryConnector = HttpsConnector::new(4, handle).unwrap();

    let secondaryClient = Client::configure()
        .connector(secondaryConnector)
        .build(handle);

    let mut queryer = Queryer {
        client: secondaryClient,
        core: c2
    };

    loop {
        let ui_rx_b = &ui_rx;
        chan_select! {
            twete_rx.recv() -> twete => match twete {
                Some(line) => {
                    let jsonstr = std::str::from_utf8(&line).unwrap().trim();
//                            println!("{}", jsonstr);
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
                            Some(line) => handle_user_input(line, &mut tweeter, &mut queryer),
                            None => std::process::exit(0)
                        }
                    }
                }
            },
            ui_rx.recv() -> user_input => match user_input {
                Some(line) => {
                    handle_user_input(line, &mut tweeter, &mut queryer);
                },
                None => println!("UI thread hung up...")
            }
        }
    }

    println!("Bye bye");
}

fn handle_user_input(line: Vec<u8>, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer) {
    if line == String::from("show_cache\n").into_bytes() {
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
    } else if line == String::from("q\n").into_bytes() {
        println!("Bye bye!");
        tweeter.store_cache();
        std::process::exit(0);
    } else if line.starts_with("look_up_".as_bytes()) {
        let linestr = std::str::from_utf8(&line).unwrap().trim();
        if linestr.starts_with("look_up_tweet") {
            let tweetid = &linestr.split(" ").collect::<Vec<&str>>()[1].to_string();
            if let Some(tweet) = tweeter.fetch_tweet(tweetid, &mut queryer) {
                println!("{:?}", tweet);
            } else {
//                                        println!("Couldn't retrieve {}", tweetid);
            }
        } else if linestr.starts_with("look_up_user") {
            let userid = &linestr.split(" ").collect::<Vec<&str>>()[1].to_string();
            if let Some(user) = tweeter.fetch_user(userid, &mut queryer) {
                println!("{:?}", user);
            } else {
//                                        println!("Couldn't retrieve {}", userid);
            }
        }
    }
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

        let req = signed_api_get(streamurl);
        let work = client.request(req).and_then(|res| {
            let status = res.status();
            if status != hyper::StatusCode::Ok {
                println!("Twitter stream connect was abnormal: {}", status);
            }
            LineStream::new(res.body()
                .map(|chunk| futures::stream::iter(chunk.into_iter().map(|b| Ok(b))))
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
            Ok(good) => (),
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

