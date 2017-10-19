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

use display::Render;
use display;

pub mod tweet;
use self::tweet::Tweet;
pub mod user;
use self::user::User;

pub fn full_twete_text(twete: &serde_json::map::Map<String, serde_json::Value>) -> String {
    if twete.contains_key("retweeted_status") {
        return full_twete_text(twete["retweeted_status"].as_object().unwrap())
    }
    let mut twete_text: String;
    twete_text = if twete["truncated"].as_bool().unwrap() {
        let extended = twete.get("extended_tweet");
        match extended {
            Some(extended_tweet) => {
                let full_text = extended_tweet.get("full_text");
                match full_text {
                    Some(text) => {
                        return text.as_str().unwrap().to_string()
                    }
                    None => {
                        println!("Missing extended_tweet text. Full extended_tweet json: {:?}", full_text);
                    }
                }
            },
            None => {
                println!("Missing extended text. Full tweet json: {:?}", twete);
            }
        }
        panic!("API bug? changed?");
    } else {
        match twete.get("text") {
            Some(text) => text.as_str().unwrap().to_string(),
            None => {
                // fall back to it maybe being at full_text...
                match twete.get("full_text") {
                    Some(text) => text.as_str().unwrap().to_string(),
                    None => panic!("api bug? text not present? text: {:?}", twete)
                }
            }
        }
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
    threads: HashMap<String, u64>, // thread : latest_tweet_in_thread
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
            current_user: User::default(),
            threads: HashMap::new()
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
            Some("quoted_tweet") => {
                self.cache_api_tweet(json["target_object"].clone());
                self.cache_api_user(json["source"].clone());
            },
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
            Some("retweeted_retweet") => {
                self.cache_api_tweet(json["target_object"].clone());
                self.cache_api_user(json["source"].clone());
                self.cache_api_user(json["target"].clone());
            },
            Some("favorited_retweet") => {
                self.cache_api_tweet(json["target_object"].clone());
                self.cache_api_user(json["source"].clone());
                self.cache_api_user(json["target"].clone());
            },
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
        let url = &format!("{}&id={}", ::TWEET_LOOKUP_URL, id);
        queryer.do_api_get(url)
    }

    pub fn get_settings(&self, queryer: &mut ::Queryer) -> Option<serde_json::Value> {
        queryer.do_api_get(::ACCOUNT_SETTINGS_URL)
    }

    pub fn get_followers(&self, queryer: &mut ::Queryer) -> Option<serde_json::Value> {
        queryer.do_api_get(::GET_FOLLOWER_IDS_URL)
    }

    pub fn set_thread(&mut self, name: String, last_id: u64) -> bool {
        self.threads.insert(name, last_id);
        true
    }

    pub fn update_thread(&mut self, name: String, last_id: u64) -> bool {
        // ensure that last_id is threaded tweet from the last one stored by name.
        // if no thread is stored by name, just use last_id.
        // who'm am'st i kid'ing, just store it for now lol
        self.threads.insert(name, last_id);
        true
    }

    pub fn latest_in_thread(&self, name: String) -> Option<&u64> {
        self.threads.get(&name)
    }

    pub fn forget_thread(&mut self, name: String) {
        self.threads.remove(&name);
    }

    /*
     * can the return type here be Iterator<
     */
    pub fn threads<'a>(&'a self) -> Box<Iterator<Item=&String> + 'a> {
        Box::new(self.threads.keys())
    }
}

fn handle_twitter_event(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    mut queryer: &mut ::Queryer) {
    tweeter.cache_api_event(structure.clone(), &mut queryer);
    if let Some(event) = events::Event::from_json(structure) {
        event.render(&tweeter);
    };
}

fn handle_twitter_delete(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    _queryer: &mut ::Queryer) {
    events::Event::Deleted {
        user_id: structure["delete"]["status"]["user_id_str"].as_str().unwrap().to_string(),
        twete_id: structure["delete"]["status"]["id_str"].as_str().unwrap().to_string()
    }.render(tweeter);
}

fn handle_twitter_twete(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    _queryer: &mut ::Queryer) {
    let twete_id = structure["id_str"].as_str().unwrap().to_string();
    tweeter.cache_api_tweet(serde_json::Value::Object(structure));
    display::render_twete(&twete_id, tweeter);
}

fn handle_twitter_dm(
    structure: serde_json::Map<String, serde_json::Value>,
    _tweeter: &mut TwitterCache,
    _queryer: &mut ::Queryer) {
    // show DM
    println!("{}", structure["direct_message"]["text"].as_str().unwrap());
    println!("Unknown struture {:?}", structure);
}

fn handle_twitter_welcome(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    queryer: &mut ::Queryer) {
//        println!("welcome: {:?}", structure);
    let user_id_nums = structure["friends"].as_array().unwrap();
    let user_id_strs = user_id_nums.into_iter().map(|x| x.as_u64().unwrap().to_string());
    tweeter.set_following(user_id_strs.collect());
    let settings = tweeter.get_settings(queryer).unwrap();
    let maybe_my_name = settings["screen_name"].as_str();
    if let Some(my_name) = maybe_my_name {
        tweeter.current_user = User {
            id: "".to_string(),
            handle: my_name.to_owned(),
            name: my_name.to_owned()
        };
        println!("You are {}", tweeter.current_user.handle);
    } else {
        println!("Unable to make API call to figure out who you are...");
    }
    let followers = tweeter.get_followers(queryer).unwrap();
    let id_arr: Vec<String> = followers["ids"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_owned()).collect();
    tweeter.set_followers(id_arr);
}

pub fn handle_message(
    twete: serde_json::Value,
    tweeter: &mut TwitterCache,
    queryer: &mut ::Queryer
) {
    match twete {
        serde_json::Value::Object(objmap) => {
            if objmap.contains_key("event") {
                handle_twitter_event(objmap, tweeter, queryer);
            } else if objmap.contains_key("friends") {
                handle_twitter_welcome(objmap, tweeter, queryer);
            } else if objmap.contains_key("delete") {
                handle_twitter_delete(objmap, tweeter, queryer);
            } else if objmap.contains_key("user") && objmap.contains_key("id") {
                handle_twitter_twete(objmap, tweeter, queryer);
            } else if objmap.contains_key("direct_message") {
                handle_twitter_dm(objmap, tweeter, queryer);
            }
            println!("");
        },
        _ => ()
    };
}
