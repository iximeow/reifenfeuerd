use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
extern crate chrono;

use self::chrono::prelude::*;

use std::collections::{HashMap, HashSet};
extern crate serde_json;
use std::io::Write;

use std::fs::OpenOptions;

pub mod events;

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
    pub fn get_mentions(&self) -> Vec<&str> {
        self.text.split(&[
            ',', '.', '/', ';', '\'',
            '[', ']', '\\', '~', '!',
            '@', '#', '$', '%', '^',
            '&', '*', '(', ')', '-',
            '=', '{', '}', '|', ':',
            '"', '<', '>', '?', '`',
            ' ' // forgot this initially. awkward.
        ][..])
            .filter(|x| x.starts_with("@") && x.len() > 1)
            .collect()
    }

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
