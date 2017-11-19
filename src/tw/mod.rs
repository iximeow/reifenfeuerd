use std::path::Path;
use std::fmt;
use std::str::FromStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::error::Error;
extern crate chrono;

use self::chrono::prelude::*;

use std::collections::{HashMap, HashSet};
extern crate serde_json;
use std::io::Write;

use std::fs::OpenOptions;

pub mod events;

use display;

pub mod tweet;
use self::tweet::Tweet;
pub mod user;
use self::user::User;

pub enum AppState {
    Shutdown,
    ShowHelp,
    Reconnect,
    Compose,
    View
}

impl Default for AppState {
    fn default() -> AppState { AppState::View }
}

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
                        text.as_str().unwrap().to_string()
                    }
                    None => {
                        panic!("Missing extended_tweet text. Full extended_tweet json: {:?}", full_text);
                    }
                }
            },
            None => {
                panic!("Missing extended text. Full tweet json: {:?}", twete);
            }
        }
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

    twete_text
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Credential {
    pub key: String,
    pub secret: String
}

#[derive(Serialize, Deserialize)]
pub struct TwitterCache {
    #[serde(skip)]
    pub users: HashMap<String, User>,
    #[serde(skip)]
    pub tweets: HashMap<String, Tweet>,
    #[serde(skip)]
    pub WIP_auth: Option<Credential>,
    pub app_key: Credential,
    // right now we're stuck assuming one profile.
    // alts and such will be others here.
    pub curr_profile: Option<String>,
    pub profiles: HashMap<String, TwitterProfile>,
    threads: HashMap<String, u64>, // thread : latest_tweet_in_thread
    #[serde(skip)]
    pub needs_save: bool,
    #[serde(skip)]
    pub caching_permitted: bool,
    #[serde(skip)]
    id_conversions: IdConversions,
    #[serde(skip)]
    pub display_info: display::DisplayInfo,
    #[serde(skip)]
    pub state: AppState
}

// Internally, a monotonically increasin i64 is always the id used.
// This is the client's id, not the twitter id for a tweet.
//
// Id forms:
//   num            // implicitly today:num
//   today:num      // last_tweet_of_yesterday_id + num
//   20171009:num   // last_tweet_of_previous_day_id + num
//   open question: what about 20171009:123451234 .. does this reference a future tweet? should
//   probably cause an issue if this would overflow into the next day.
//   ::num          // tweet id num
//   twitter::num   // twiter tweet id num
struct IdConversions {
    // maps a day to the base id for tweets off that day.
    id_per_date: HashMap<String, u64>,
    id_to_tweet_id: HashMap<u64, String>
    // twitter id to id is satisfied by looking up the twitter id in tweeter.tweets and getting
    // .inner_id
}

impl Default for IdConversions {
    fn default() -> Self {
        IdConversions {
            id_per_date: HashMap::new(),
            id_to_tweet_id: HashMap::new()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TweetId {
    Today(u64),         // just a number
    Dated(String, u64), // 20171002:number
    Bare(u64),          // ::number
    Twitter(String)     // twitter::number
}

impl fmt::Display for TweetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TweetId::Today(ref id) => {
                write!(f, "{}", id)
            },
            &TweetId::Dated(ref date, ref id) => {
                write!(f, "{}:{}", date, id)
            },
            &TweetId::Bare(ref id) => {
                write!(f, ":{}", id)
            },
            &TweetId::Twitter(ref id) => {
                write!(f, "twitter:{}", id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tweet_id_parse_test() {
        assert_eq!(TweetId::parse("12345".to_string()), Some(TweetId::Today(12345)));
        assert_eq!(TweetId::parse("20170403:12345".to_string()), Some(TweetId::Dated("20170403".to_string(), 12345)));
        assert_eq!(TweetId::parse(":12345".to_string()), Some(TweetId::Bare(12345)));
        assert_eq!(TweetId::parse("twitter:12345".to_string()), Some(TweetId::Twitter("12345".to_string())));
        assert_eq!(TweetId::parse("twitter:asdf".to_string()), Some(TweetId::Twitter("asdf".to_string())));
        assert_eq!(TweetId::parse("a2345".to_string()), None);
        assert_eq!(TweetId::parse(":".to_string()), None);
        assert_eq!(TweetId::parse("::".to_string()), None);
        assert_eq!(TweetId::parse("a:13234".to_string()), None);
        assert_eq!(TweetId::parse(":a34".to_string()), None);
        assert_eq!(TweetId::parse("asdf:34".to_string()), None);
    }
}

impl TweetId {
    pub fn parse(id_str: String) -> Result<TweetId, String> {
        // TODO: figure out how to return a Result<TweetId, <.. the result types ..>>
        if id_str.starts_with("twitter:") {
            Ok(TweetId::Twitter(id_str.chars().skip("twitter:".len()).collect()))
        } else if id_str.starts_with(":") {
            let rest = id_str.chars().skip(1);
            u64::from_str(&rest.collect::<String>())
                .map(TweetId::Bare)
                .map_err(|err| err.description().to_string())
        } else if id_str.find(":") == Some(8) {
            let strings: Vec<&str> = id_str.split(":").collect();
            if strings.len() == 2 && strings[0].chars().all(|x| x.is_digit(10)) {
                u64::from_str(strings[1])
                    .map(|id| TweetId::Dated(strings[0].to_owned(), id))
                    .map_err(|err| err.description().to_string())
            } else {
                Err("Invalid format, date and id must be all numbers".to_string())
            }
        } else if id_str.chars().all(|x| x.is_digit(10)) {
            // today
            u64::from_str(&id_str)
                .map(TweetId::Today)
                .map_err(|err| err.description().to_string())
        } else {
            Err(format!("Unrecognized id string: {}", id_str))
        }
    }
}

impl IdConversions {
    // So this is twid -> Option<u64>
    // elsewhere we u64 -> Option<Tweet>
    //
    // except in the TweetId::Twitter case we TweetId -> Option<Tweet> -> Option<u64> ... to ->
    // Option<Tweet> in the future?
    //                          // WHY must we take mutable borrow of TwitterCache here, you ask?
    //                          // well, because it contains display_info, and retrieve_tweet can
    //                          // end up logging, for now!
    fn to_inner_id(&self, tweeter: &mut TwitterCache, twid: TweetId) -> Option<u64> {
        match twid {
            TweetId::Today(num) => {
                let first_for_today: u64 = 0;
                Some(first_for_today + num)
            },
            TweetId::Dated(date, num) => {
                let first_for_date: u64 = 0;
                Some(first_for_date + num)
            },
            TweetId::Bare(num) => Some(num),
            twid @ TweetId::Twitter(_) => tweeter.retrieve_tweet(&twid).map(|x| x.internal_id)
        }
    }
}

use commands::Command;
use Queryer;

// TODO:
// is there a nice way to make this accept commands: Iterable<&'a Command>? eg either a Vec or an
// array or whatever?
// (extra: WITHOUT having to build an iterator?)
// ((extra 2: when compiled with -O3, how does `commands` iteration look? same as array?))
fn parse_word_command<'a, 'b>(line: &'b str, commands: &[&'a Command]) -> Option<(&'b str, &'a Command)> {
    for cmd in commands.into_iter() {
        if cmd.params == 0 {
            if line == cmd.keyword {
                return Some(("", &cmd));
            }
        } else if line.starts_with(cmd.keyword) {
            if line.find(" ").map(|x| x == cmd.keyword.len()).unwrap_or(false) {
                return Some((line.get((cmd.keyword.len() + 1)..).unwrap().trim(), &cmd));
            }
        }
    }
    return None
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TwitterProfile {
    pub creds: Credential,
    pub user: User,
    following: HashSet<String>,
    following_history: HashMap<String, (String, i64)>, // userid:date??
    pub followers: HashSet<String>,
    lost_followers: HashSet<String>,
    follower_history: HashMap<String, (String, i64)> // userid:date??
}

impl TwitterProfile {
    pub fn new(creds: Credential, user: User) -> TwitterProfile {
        TwitterProfile {
            creds: creds,
            user: user,
            following: HashSet::new(),
            following_history: HashMap::new(),
            followers: HashSet::new(),
            lost_followers: HashSet::new(),
            follower_history: HashMap::new()
        }
    }
}

impl TwitterCache {
    const PROFILE_DIR: &'static str = "cache/";
    const TWEET_CACHE: &'static str = "cache/tweets.json";
    const USERS_CACHE: &'static str = "cache/users.json";
    const PROFILE_CACHE: &'static str = "cache/profile.json";

    fn new() -> TwitterCache {
        TwitterCache {
            users: HashMap::new(),
            tweets: HashMap::new(),
            WIP_auth: None,
            app_key: Credential {
                key: "".to_owned(),
                secret: "".to_owned()
            },
            // So, supporting multiple profiles will be ... interesting?
            // how do we support a variable number of channels? which will be necessary as we'll
            // have one channel up per twitter stream...
            curr_profile: None,
            profiles: HashMap::new(),
            needs_save: false,
            caching_permitted: true,
            threads: HashMap::new(),
            id_conversions: IdConversions::default(),
            display_info: display::DisplayInfo::default(),
            state: AppState::View
        }
    }

    pub fn current_profile(&self) -> Option<&TwitterProfile> {
        match &self.curr_profile {
            &Some(ref profile_name) => self.profiles.get(profile_name),
            &None => None
        }
    }

    pub fn current_handle(&self) -> Option<String> {
        self.current_profile().map(|profile| profile.user.handle.to_owned())
    }

    // TODO: pull out the "Cache" part of TwitterCache, it can be serialized/deserialized - the
    // rest of the history is just for the running instance..
    pub fn handle_user_input(&mut self, line: Vec<u8>, mut queryer: &mut Queryer) {
        let command_bare = String::from_utf8(line).unwrap();
        let command = command_bare.trim();
        if let Some((line, cmd)) = parse_word_command(&command, ::commands::COMMANDS) {
            (cmd.exec)(line.to_owned(), self, &mut queryer);
        } else {
            self.display_info.status(format!("I don't know what {} means", command).to_string());
        }
    }

    fn new_without_caching() -> TwitterCache {
        let mut cache = TwitterCache::new();
        cache.caching_permitted = false;
        cache
    }
    pub fn add_profile(&mut self, profile: TwitterProfile, name: Option<String>) {
        self.profiles.insert(name.unwrap_or(profile.user.handle.to_owned()), profile);
        if self.caching_permitted {
            self.store_cache();
        }
    }
    fn cache_user(&mut self, user: User) {
        let update_cache = match self.users.get(&user.id) {
            Some(cached_user) => &user != cached_user,
            None => true
        };

        if update_cache {
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
    pub fn store_cache(&mut self) {
        if self.caching_permitted {
            if Path::new(TwitterCache::PROFILE_DIR).is_dir() {
                let profile = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(false)
                    .truncate(true) // since this one can become smaller, lop off trailing characters
                    .open(TwitterCache::PROFILE_CACHE)
                    .unwrap();
                serde_json::to_writer(profile, self).unwrap();
            } else {
                self.display_info.status("No cache dir exists...".to_owned());
            }
        }
    }
    fn number_and_insert_tweet(&mut self, mut tw: Tweet) {
        if !self.tweets.contains_key(&tw.id.to_owned()) {
            if tw.internal_id == 0 {
                tw.internal_id = (self.tweets.len() as u64) + 1;
                self.id_conversions.id_to_tweet_id.insert(tw.internal_id, tw.id.to_owned());
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
                                let unwrapped_line = line.unwrap();
                                let t: Result<Tweet, serde_json::Error> = serde_json::from_str(&unwrapped_line.clone());
                                match t {
                                    Ok(tweet) => cache.number_and_insert_tweet(tweet),
                                    Err(e) => panic!(format!("{} on line {} - {:?}", e, cache.tweets.len(), unwrapped_line.clone()))
                                };
                            }
                            for line in BufReader::new(File::open(TwitterCache::USERS_CACHE).unwrap()).lines() {
                                let unwrapped_line = line.unwrap();
                                let u: Result<User, serde_json::Error> = serde_json::from_str(&unwrapped_line.clone());
                                match u {
                                    Ok(user) => cache.users.insert(user.id.to_owned(), user),
                                    Err(e) => panic!(format!("{} on line {} - {:?}", e, cache.users.len(), unwrapped_line.clone()))
                                };
                            }
                            cache.caching_permitted = true;
                            cache.needs_save = false;
                            cache
                        }
                        Err(e) => {
                            // TODO! should be able to un-frick profile after startup.
                            let mut cache = TwitterCache::new_without_caching();
                            cache.display_info.status(format!("Error reading profile, profile caching disabled... {}", e));
                            cache
                        }
                    }
                }
                Err(e) => {
                    let mut cache = TwitterCache::new_without_caching();
                    cache.display_info.status(format!("Error reading cached profile: {}. Profile caching disabled.", e));
                    cache
                }
            }
        } else {
            let mut cache = TwitterCache::new();
            cache.display_info.status(format!("Hello! First time setup?"));
            cache
        }
    }
    pub fn cache_api_tweet(&mut self, json: serde_json::Value) {
        // TODO: log error somehow
        if let Some(Ok((rt, rt_user))) = json.get("retweeted_status").map(|x| Tweet::from_api_json(x.to_owned())) {
            self.cache_user(rt_user);
            self.cache_tweet(rt);
        }

        // TODO: log error somehow
        if let Some(Ok((qt, qt_user))) = json.get("quoted_status").map(|x| Tweet::from_api_json(x.to_owned())) {
            self.cache_user(qt_user);
            self.cache_tweet(qt);
        }

        // TODO: log error somehow
        if let Ok((twete, user)) = Tweet::from_api_json(json) {
            self.cache_user(user);
            self.cache_tweet(twete);
        }
    }
    pub fn cache_api_user(&mut self, json: serde_json::Value) {
        // TODO: log error somehow
        // TODO: probably means display_info needs a more technical-filled log for debugging,
        //       independent of the user-facing statuses, like "invalid id"
        if let Ok(user) = User::from_json(json) {
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
    pub fn retrieve_tweet(&mut self, tweet_id: &TweetId) -> Option<&Tweet> {
        match tweet_id {
            &TweetId::Bare(ref id) => {
                let maybe_tweet_id = self.id_conversions.id_to_tweet_id.get(id);
                match maybe_tweet_id {
                    Some(id) => self.tweets.get(id),
                    None => None
                }
            },
            &TweetId::Today(ref id) => {
                let inner_id = self.id_conversions.id_to_tweet_id.get(id);
                self.display_info.status("Retrieving tweets with dated IDs is not yet supported.".to_string());
                None
            },
            &TweetId::Dated(ref date, ref id) => {
                self.display_info.status("Retrieving tweets with dated IDs is not yet supported.".to_string());
                None
            },
            &TweetId::Twitter(ref id) => self.tweets.get(id)
        }
    }
    pub fn retrieve_user(&self, user_id: &String) -> Option<&User> {
        self.users.get(user_id)
    }
    pub fn fetch_tweet(&mut self, tweet_id: &TweetId, mut queryer: &mut ::Queryer) -> Option<&Tweet> {
        match tweet_id {
            &TweetId::Bare(ref id) => {
                // we can do nothing but just try to get it
                self.retrieve_tweet(tweet_id)
            }
            &TweetId::Today(ref id) => {
                // we can do nothing but just try to get it
                self.retrieve_tweet(tweet_id)
            },
            &TweetId::Dated(ref date, ref id) => {
                // we can do nothing but just try to get it
                self.retrieve_tweet(tweet_id)
            },
            &TweetId::Twitter(ref id) => {
                if !self.tweets.contains_key(id) {
                    match self.look_up_tweet(id, &mut queryer) {
                        Ok(json) => self.cache_api_tweet(json),
                        Err(e) => self.display_info.status(format!("Unable to retrieve tweet {}:\n{}", id, e))
                    };
                }
                self.retrieve_tweet(tweet_id)
            }
        }
    }
    pub fn fetch_user(&mut self, user_id: &String, mut queryer: &mut ::Queryer) -> Option<&User> {
        if !self.users.contains_key(user_id) {
            let maybe_parsed = self.look_up_user(user_id, &mut queryer).and_then(|x| User::from_json(x));
            match maybe_parsed {
                Ok(tw) => self.cache_user(tw),
                Err(e) => self.display_info.status(format!("Unable to retrieve user {}:\n{}", user_id, e))
            }
        }
        self.users.get(user_id)
    }
    pub fn set_following(&mut self, user_ids: Vec<String>) {
        self.current_profile().map(|profile| profile.to_owned()).map(|mut profile| {
            let uid_set = user_ids.into_iter().collect::<HashSet<String>>();

            let new_uids = &uid_set - &profile.following;
            for user in &new_uids {
                self.display_info.status(format!("New following! {}", user));
                self.add_following(user);
            }

            let lost_uids = &profile.following - &uid_set;
            for user in &lost_uids {
                self.display_info.status(format!("Bye, friend! {}", user));
                self.remove_following(user);
            }
        });
    }
    pub fn set_followers(&mut self, user_ids: Vec<String>) {
        self.current_profile().map(|profile| profile.to_owned()).map(|mut profile| {
            let uid_set = user_ids.into_iter().collect::<HashSet<String>>();

            let new_uids = &uid_set - &profile.followers;
            for user in &new_uids {
                self.display_info.status(format!("New follower! {}", user));
                self.add_follower(user);
            }

            let lost_uids = &profile.followers - &uid_set;
            for user in &lost_uids {
                self.display_info.status(format!("Bye, friend! {}", user));
                self.remove_follower(user);
            }
        });
    }
    pub fn add_following(&mut self, user_id: &String) {
        self.current_profile().map(|profile| profile.to_owned()).map(|mut profile| {
            self.needs_save = true;
            profile.following.insert(user_id.to_owned());
            profile.following_history.insert(user_id.to_owned(), ("following".to_string(), Utc::now().timestamp()));
        });
    }
    pub fn remove_following(&mut self, user_id: &String) {
        self.current_profile().map(|profile| profile.to_owned()).map(|mut profile| {
            self.needs_save = true;
            profile.following.remove(user_id);
            profile.following_history.insert(user_id.to_owned(), ("unfollowing".to_string(), Utc::now().timestamp()));
        });
    }
    pub fn add_follower(&mut self, user_id: &String) {
        self.current_profile().map(|profile| profile.to_owned()).map(|mut profile| {
            self.needs_save = true;
            profile.followers.insert(user_id.to_owned());
            profile.lost_followers.remove(user_id);
            profile.follower_history.insert(user_id.to_owned(), ("follow".to_string(), Utc::now().timestamp()));
        });
    }
    pub fn remove_follower(&mut self, user_id: &String) {
        self.current_profile().map(|profile| profile.to_owned()).map(|mut profile| {
            self.needs_save = true;
            profile.followers.remove(user_id);
            profile.lost_followers.insert(user_id.to_owned());
            profile.follower_history.insert(user_id.to_owned(), ("unfollow".to_string(), Utc::now().timestamp()));
        });
    }

    fn look_up_user(&mut self, id: &str, queryer: &mut ::Queryer) -> Result<serde_json::Value, String> {
        let url = &format!("{}?user_id={}", ::USER_LOOKUP_URL, id);
        match self.current_profile() {
            Some(ref user_profile) => queryer.do_api_get(url, &self.app_key, &user_profile.creds),
            None => Err("No authorized user to conduct lookup".to_owned())
        }
    }

    fn look_up_tweet(&mut self, id: &str, queryer: &mut ::Queryer) -> Result<serde_json::Value, String> {
        let url = &format!("{}&id={}", ::TWEET_LOOKUP_URL, id);
        match self.current_profile() {
            Some(ref user_profile) => queryer.do_api_get(url, &self.app_key, &user_profile.creds),
            None => Err("No authorized user to conduct lookup".to_owned())
        }
    }

    pub fn get_settings(&self, queryer: &mut ::Queryer) -> Result<serde_json::Value, String> {
        match self.current_profile() {
            Some(ref user_profile) => queryer.do_api_get(::ACCOUNT_SETTINGS_URL, &self.app_key, &user_profile.creds),
            None => Err("No authorized user to request settings".to_owned())
        }
    }

    pub fn get_followers(&self, queryer: &mut ::Queryer) -> Result<serde_json::Value, String> {
        match self.current_profile() {
            Some(ref user_profile) => queryer.do_api_get(::GET_FOLLOWER_IDS_URL, &self.app_key, &user_profile.creds),
            None => Err("No authorized user to request followers".to_owned())
        }
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
    match events::Event::from_json(structure) {
        Ok(event) => {
            tweeter.display_info.recv(display::Infos::Event(event));
        },
        Err(e) => {
            tweeter.display_info.status(format!("Unknown twitter json: {:?}", e));
        }
    }
}

fn handle_twitter_delete(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    _queryer: &mut ::Queryer) {
    /*
    tweeter.display_info.recv(display::Infos::Event(
        events::Event::Deleted {
            user_id: structure["delete"]["status"]["user_id_str"].as_str().unwrap().to_string(),
            twete_id: structure["delete"]["status"]["id_str"].as_str().unwrap().to_string()
        }
    ));
    */
}

fn handle_twitter_twete(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    _queryer: &mut ::Queryer) {
    let twete_id = structure["id_str"].as_str().unwrap().to_string();
    tweeter.cache_api_tweet(serde_json::Value::Object(structure));
    tweeter.display_info.recv(display::Infos::Tweet(TweetId::Twitter(twete_id)));
    // display::render_twete(&twete_id, tweeter);
}

fn handle_twitter_dm(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    _queryer: &mut ::Queryer) {
    // show DM
    tweeter.display_info.recv(display::Infos::DM(structure["direct_message"]["text"].as_str().unwrap().to_string()));
}

fn handle_twitter_welcome(
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    queryer: &mut ::Queryer) {
    let user_id_nums = structure["friends"].as_array().unwrap();
    let user_id_strs = user_id_nums.into_iter().map(|x| x.as_u64().unwrap().to_string());
    tweeter.set_following(user_id_strs.collect());
    let settings = tweeter.get_settings(queryer).unwrap();
    let maybe_my_name = settings["screen_name"].as_str();
    if let Some(my_name) = maybe_my_name {
        // TODO: come back to this when custom profile names are supported?
        tweeter.curr_profile = Some(my_name.to_owned());
        tweeter.display_info.status(format!("You are {}", my_name))
    } else {
        tweeter.display_info.status("Unable to make API call to figure out who you are...".to_string());
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
            } else {
                tweeter.display_info.status(format!("Unknown json: {:?}", objmap));
            }
//            self.display_info.status("");
        },
        _ => ()
    };
}
