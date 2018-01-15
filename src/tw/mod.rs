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

use display::DisplayInfo;

#[derive(Clone)]
pub enum AppState {
    Shutdown,
    ShowHelp,
    Reconnect(String),
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
enum TweetMuteType {
    Notifications,
    NotificationsAndReplies,
    Conversation
}

#[derive(Serialize, Deserialize)]
enum UserMuteType {
    Retweets,
    Mentions,
    Everything
}

#[derive(Serialize, Deserialize)]
struct MuteInfo {
    pub users: HashMap<String, UserMuteType>, // user id strings : mute info
    pub tweets: HashMap<String, TweetMuteType>, // twitter tweet id : mute info
    pub words: HashSet<String>
}

impl MuteInfo {
    fn new() -> MuteInfo {
        MuteInfo {
            users: HashMap::new(),
            tweets: HashMap::new(),
            words: HashSet::new()
        }
    }
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
    mutes: MuteInfo,
    threads: HashMap<String, u64>, // thread : latest_tweet_in_thread
    #[serde(skip)]
    pub needs_save: bool,
    #[serde(skip)]
    pub caching_permitted: bool,
    #[serde(skip)]
    id_conversions: IdConversions,
//    #[serde(skip)]
//    pub display_info: display::DisplayInfo,
    #[serde(skip)]
    pub state: AppState,
    #[serde(skip)]
    pub connection_map: HashMap<u8, String>
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
    id_to_tweet_id: HashMap<u64, String>,
    // twitter id to id is satisfied by looking up the twitter id in tweeter.tweets and getting
    // .inner_id
    // YYYYMMDD : day_id : inner_tweet_id
    tweets_by_date: HashMap<String, HashMap<u64, u64>>,
    // YYYYMMDD : inner_tweet_id : day_id
    tweets_by_date_and_tweet_id: HashMap<String, HashMap<u64, u64>>
}

impl Default for IdConversions {
    fn default() -> Self {
        IdConversions {
            id_to_tweet_id: HashMap::new(),
            tweets_by_date: HashMap::new(),
            tweets_by_date_and_tweet_id: HashMap::new()
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
        assert_eq!(TweetId::parse("12345".to_string()), Ok(TweetId::Today(12345)));
        assert_eq!(TweetId::parse("20170403:12345".to_string()), Ok(TweetId::Dated("20170403".to_string(), 12345)));
        assert_eq!(TweetId::parse("20170403:12345".to_string()), Ok(TweetId::Dated("20170403".to_string(), 12345)));
        assert_eq!(TweetId::parse(":12345".to_string()), Ok(TweetId::Bare(12345)));
        assert_eq!(TweetId::parse("twitter:12345".to_string()), Ok(TweetId::Twitter("12345".to_string())));
        assert_eq!(TweetId::parse("twitter:asdf".to_string()), Ok(TweetId::Twitter("asdf".to_string())));
        assert_eq!(TweetId::parse("a2345".to_string()), Err("Unrecognized id string: a2345".to_owned()));
        // TODO: clarify
        assert_eq!(TweetId::parse(":".to_string()), Err("cannot parse integer from empty string".to_owned()));
        // TODO: clarify
        assert_eq!(TweetId::parse("::".to_string()), Err("invalid digit found in string".to_owned()));
        assert_eq!(TweetId::parse("a:13234".to_string()), Err("Unrecognized id string: a:13234".to_owned()));
        assert_eq!(TweetId::parse(":a34".to_string()), Err("invalid digit found in string".to_owned()));
        assert_eq!(TweetId::parse("asdf:34".to_string()), Err("Unrecognized id string: asdf:34".to_owned()));
    }

    #[test]
    fn test_mute_behavior() {
        let today = Utc::now();
        let rt_muted_user = "1";
        let everything_muted_user = "2";
        let you = "3";
        let rando = "4";

        let you_tweet = Tweet {
            id: "yours".to_owned(),
            author_id: you.to_owned(),
            text: "you said something".to_owned(),
            created_at: "todayish".to_owned(),
            recieved_at: today,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: None,
            reply_to_tweet: None,
            internal_id: 0
        };

        let rt_muted_user_reply = Tweet {
            id: "rt_muted_user_reply".to_owned(),
            author_id: rt_muted_user.to_owned(),
            text: "i said something in reply".to_owned(),
            created_at: "todayish".to_owned(),
            recieved_at: today,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: None,
            reply_to_tweet: Some("yours".to_owned()),
            internal_id: 0
        };

        let everything_muted_user_reply = Tweet {
            id: "everything_muted_user_reply".to_owned(),
            author_id: everything_muted_user.to_owned(),
            text: "i also said something in reply".to_owned(),
            created_at: "todayish".to_owned(),
            recieved_at: today,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: None,
            reply_to_tweet: Some("yours".to_owned()),
            internal_id: 0
        };

        let rando_reply = Tweet {
            id: "rando reply".to_owned(),
            author_id: rando.to_owned(),
            text: "some random reply".to_owned(),
            created_at: "todayish".to_owned(),
            recieved_at: today,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: None,
            reply_to_tweet: Some("yours".to_owned()),
            internal_id: 0
        };

        let rt_of_rando_reply = Tweet {
            id: "rt_of_rando_reply".to_owned(),
            author_id: rt_muted_user.to_owned(),
            text: "rando reply text".to_owned(),
            created_at: "todayish".to_owned(),
            recieved_at: today,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: Some(rando_reply.id.to_owned()),
            reply_to_tweet: Some("yours".to_owned()),
            internal_id: 0
        };

        let muted_rt_of_you = Tweet {
            id: "rt_of_yours".to_owned(),
            author_id: rt_muted_user.to_owned(),
            text: "you said something".to_owned(),
            created_at: "todayish".to_owned(),
            recieved_at: today,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: Some("yours".to_owned()),
            reply_to_tweet: None,
            internal_id: 0
        };

        let tweets = vec![
            &you_tweet, &rt_muted_user_reply, &everything_muted_user_reply, &rando_reply,
            &rt_of_rando_reply, &muted_rt_of_you,
        ];

        let mut tweeter = TwitterCache::new();

        for tweet in tweets {
            tweeter.number_and_insert_tweet(tweet.to_owned());
        }

        tweeter.mute_user(rt_muted_user.to_owned(), UserMuteType::Retweets);
        tweeter.mute_user(everything_muted_user.to_owned(), UserMuteType::Everything);

        assert_eq!(tweeter.tweet_muted(&you_tweet), false);
        assert_eq!(tweeter.tweet_muted(&rt_muted_user_reply), true);
        assert_eq!(tweeter.tweet_muted(&everything_muted_user_reply), true);
        assert_eq!(tweeter.tweet_muted(&rando_reply), false);
        assert_eq!(tweeter.tweet_muted(&rt_of_rando_reply), true);
        assert_eq!(tweeter.tweet_muted(&muted_rt_of_you), true);
    }

    #[test]
    fn test_display_id() {
        /*
         * ... I think this test only works if Local is -0800 or further from UTC.
         * I'm not even sure how to make any of this work in a TZ-invariant way.
         */
        // THIS NOW ONLY WORKS FOR DAYS OTHER THAN THE LAST OF A MONTH. FML
        let today = Local::now();
        let pst_1630_but_utc: DateTime<Utc> = DateTime::parse_from_rfc3339(
            &format!("{:04}-{:02}-{:02}T00:30:00Z", today.year(), today.month(), today.day() + 1)).unwrap().with_timezone(&Utc);
        let local_pst_1630: DateTime<Local> = DateTime::parse_from_rfc3339(
            &format!("{:04}-{:02}-{:02}T16:30:00-08:00", today.year(), today.month(), today.day())).unwrap().with_timezone(&Local);
        let tweet = Tweet {
            id: "manual_tweet_1".to_owned(),
            author_id: "you, dummy".to_owned(),
            text: "test tweet please ignore".to_owned(),
            created_at: "1234 not real".to_owned(),
            recieved_at: pst_1630_but_utc,
            urls: HashMap::new(),
            quoted_tweet_id: None,
            rt_tweet: None,
            reply_to_tweet: None,
            internal_id: 0
        };

        let mut tweeter = TwitterCache::new();

        tweeter.number_and_insert_tweet(tweet.clone());

        let retrieved = tweeter.retrieve_tweet(&TweetId::Bare(1));
        assert_eq!(retrieved.is_some(), true);

        let retrieved = tweeter.retrieve_tweet(&TweetId::Dated(format!("{:04}{:02}{:02}", today.year(), today.month(), today.day()), 0));
        assert_eq!(retrieved.is_some(), true);

        let display_id = tweeter.display_id_for_tweet(&retrieved.unwrap());

        assert_eq!(display_id, TweetId::Today(0));
    }

    #[test]
    fn test_tweet_retrieval() {
        let today = Utc::now();
        let yesterday = today - chrono::Duration::days(1);
        let tweets = vec![
            Tweet {
                id: "manual_tweet_1".to_owned(),
                author_id: "author_1".to_owned(),
                text: "this is a test".to_owned(),
                created_at: "1234 not real lol".to_owned(),
                recieved_at: yesterday,
                urls: HashMap::new(),
                quoted_tweet_id: None,
                rt_tweet: None,
                reply_to_tweet: None,
                internal_id: 0
            },
            Tweet {
                id: "manual_tweet_2".to_owned(),
                author_id: "author_1".to_owned(),
                text: "this is a test".to_owned(),
                created_at: "1234 not real lol".to_owned(),
                recieved_at: today,
                urls: HashMap::new(),
                quoted_tweet_id: None,
                rt_tweet: None,
                reply_to_tweet: None,
                internal_id: 0
            }
        ];

        let mut tweeter = TwitterCache::new();

        for tweet in &tweets {
            tweeter.number_and_insert_tweet(tweet.to_owned());
        }

        assert_eq!(
            tweeter.retrieve_tweet(&TweetId::Twitter("manual_tweet_1".to_owned())).map(|x| x.id.to_owned()),
            Some(tweets[0].clone()).map(|x| x.id)
        );
        assert_eq!(
            tweeter.retrieve_tweet(&TweetId::Today(0)).map(|x| x.id.to_owned()),
            Some(tweets[1].clone()).map(|x| x.id)
        );

        let local_yesterday = yesterday.with_timezone(&Local);
        let date = format!("{:04}{:02}{:02}", local_yesterday.year(), local_yesterday.month(), local_yesterday.day());

        assert_eq!(
            tweeter.retrieve_tweet(&TweetId::Dated(date, 0)).map(|x| x.id.to_owned()),
            Some(tweets[0].clone()).map(|x| x.id)
        );
    }
}

impl TweetId {
    pub fn parse(id_str: String) -> Result<TweetId, String> {
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
    fn to_twitter_id(&self, twid: TweetId) -> Option<String> {
        match twid {
            TweetId::Today(num) => {
                let now = Local::now();
                let now_date_str = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day());
                let tweet_id = self.tweets_by_date.get(&now_date_str).and_then(|x| x.get(&num));
                tweet_id.and_then(|x| self.id_to_tweet_id.get(x)).map(|x| x.to_owned())
            },
            TweetId::Dated(date, num) => {
                let tweet_id = self.tweets_by_date.get(&date).and_then(|x| x.get(&num));
                tweet_id.and_then(|x| self.id_to_tweet_id.get(x)).map(|x| x.to_owned())
            },
            TweetId::Bare(num) => self.id_to_tweet_id.get(&num).map(|x| x.to_owned()),
            TweetId::Twitter(id) => Some(id)
        }
    }

    fn to_display_id(&self, twid: &TweetId, tweeter: &TwitterCache) -> TweetId {
        match twid {
            id @ &TweetId::Today(_) => id.to_owned(),
            id @ &TweetId::Dated(_, _) => {
                tweeter.retrieve_tweet(id).map(|x| TweetId::Bare(x.internal_id)).unwrap_or(id.to_owned())
            },
            id @ &TweetId::Bare(_) => {
                tweeter.retrieve_tweet(id).and_then(|tweet| {
                    let now = Local::now();
                    let tweet_date = tweet.recieved_at.with_timezone(&Local);
                    if now.year() == tweet_date.year() && now.month() == tweet_date.month() && now.day() == tweet_date.day() {
                        let date_string = format!("{:04}{:02}{:02}", tweet_date.year(), tweet_date.month(), tweet_date.day());
                        let today_id = self.tweets_by_date_and_tweet_id.get(&date_string).and_then(|m| m.get(&tweet.internal_id));
                        today_id.map(|x| TweetId::Today(*x))
                    } else {
                        Some(TweetId::Bare(tweet.internal_id))
                    }
                }).unwrap_or(id.to_owned())
            },
            id @ &TweetId::Twitter(_) => {
                tweeter.retrieve_tweet(id).and_then(|tweet| {
                    let now = Local::now();
                    let tweet_date = tweet.recieved_at.with_timezone(&Local);
                    if now.year() == tweet_date.year() && now.month() == tweet_date.month() && now.day() == tweet_date.day() {
                        let date_string = format!("{:04}{:02}{:02}", tweet_date.year(), tweet_date.month(), tweet_date.day());
                        let today_id = self.tweets_by_date_and_tweet_id.get(&date_string).and_then(|m| m.get(&tweet.internal_id));
                        today_id.map(|x| TweetId::Today(*x))
                    } else {
                        Some(TweetId::Bare(tweet.internal_id))
                    }
                }).unwrap_or(id.to_owned())
            }
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
            } else if line.starts_with(&format!("{} ", cmd.keyword)) {
                return Some((line.get((cmd.keyword.len() + 1)..).unwrap().trim(), &cmd));
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
    pub fn get_settings(&self, queryer: &mut ::Queryer, app_key: &Credential) -> Result<serde_json::Value, String> {
        queryer.do_api_get_noparam(::ACCOUNT_SETTINGS_URL, app_key, &self.creds)
    }
    pub fn get_followers(&self, queryer: &mut ::Queryer, app_key: &Credential) -> Result<serde_json::Value, String> {
        queryer.do_api_get_noparam(::GET_FOLLOWER_IDS_URL, app_key, &self.creds)
    }
    pub fn set_following(&mut self, user_ids: Vec<String>) -> (Vec<String>, Vec<String>) {
        let uid_set = user_ids.into_iter().collect::<HashSet<String>>();
        let mut new_following: Vec<String> = vec![];
        let mut lost_following: Vec<String> = vec![];

        let new_uids = &uid_set - &self.following;
        for user in new_uids {
            self.add_following(&user);
            new_following.push(user);
        }

        let lost_uids = &self.following - &uid_set;
        for user in lost_uids {
            self.remove_following(&user);
            lost_following.push(user);
        }
        (new_following, lost_following)
    }
    pub fn set_followers(&mut self, user_ids: Vec<String>) -> (Vec<String>, Vec<String>) {
        let uid_set = user_ids.into_iter().collect::<HashSet<String>>();
        let mut new_follower: Vec<String> = vec![];
        let mut lost_follower: Vec<String> = vec![];

        let new_uids = &uid_set - &self.followers;
        for user in new_uids {
            self.add_follower(&user);
            new_follower.push(user);
        }

        let lost_uids = &self.followers - &uid_set;
        for user in lost_uids {
            self.remove_follower(&user);
            lost_follower.push(user);
        }
        (new_follower, lost_follower)
    }
    /*
     * Returns: "did this change?"
     */
    pub fn add_following(&mut self, user_id: &String) -> bool {
        let mut changed = self.following.insert(user_id.to_owned());
        changed |= self.following_history.insert(user_id.to_owned(), ("following".to_string(), Utc::now().timestamp())).is_none();
        changed
    }
    /*
     * Returns: "did this change?"
     */
    pub fn remove_following(&mut self, user_id: &String) -> bool {
        let mut changed = self.following.remove(user_id);
        changed |= self.following_history.insert(user_id.to_owned(), ("unfollowing".to_string(), Utc::now().timestamp())).is_some();
        changed
    }
    /*
     * Returns: "did this change?"
     */
    pub fn add_follower(&mut self, user_id: &String) -> bool {
        let mut changed = self.followers.insert(user_id.to_owned());
        changed |= self.lost_followers.remove(user_id);
        changed |= self.follower_history.insert(user_id.to_owned(), ("follow".to_string(), Utc::now().timestamp())).is_none();
        changed
    }
    /*
     * Returns: "did this change?"
     */
    pub fn remove_follower(&mut self, user_id: &String) -> bool {
        let mut changed = self.followers.remove(user_id);
        changed |= self.lost_followers.insert(user_id.to_owned());
        changed |= self.follower_history.insert(user_id.to_owned(), ("unfollow".to_string(), Utc::now().timestamp())).is_some();
        changed
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
            state: AppState::View,
            connection_map: HashMap::new(),
            mutes: MuteInfo::new()
        }
    }

    pub fn mut_profile_for_connection(&mut self, conn_id: u8) -> &mut TwitterProfile {
        self.profiles.get_mut(&self.connection_map[&conn_id]).unwrap()
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
    pub fn handle_user_input(&mut self, line: Vec<u8>, mut queryer: &mut Queryer, display_info: &mut DisplayInfo) {
        let command_bare = String::from_utf8(line).unwrap();
        let command = command_bare.trim();
        if let Some((line, cmd)) = parse_word_command(&command, ::commands::COMMANDS) {
            (cmd.exec)(line.to_owned(), self, &mut queryer, display_info);
        } else {
            display_info.status(format!("I don't know what {} means", command).to_string());
        }
    }

    fn mute_tweet(&mut self, twid: String, mute_setting: TweetMuteType) {
        self.mutes.tweets.insert(twid, mute_setting);
    }

    fn mute_user(&mut self, userid: String, mute_setting: UserMuteType) {
        self.mutes.users.insert(userid, mute_setting);
    }

    fn unmute_tweet(&mut self, twid: String) {
        self.mutes.tweets.remove(&twid);
    }

    fn unmute_user(&mut self, userid: String) {
        self.mutes.users.remove(&userid);
    }

    fn tweet_muted(&self, tw: &Tweet) -> bool {
        match self.mutes.tweets.get(&tw.id) {
            Some(Conversation) => {
                // you may have muted a tweet directly?
                return true;
            }
            _ => {}
        };
        match self.mutes.users.get(&tw.author_id) {
            Some(Mentions) => { return true; },
            Some(Everything) => { return true; },
            Some(Retweets) => {
                // if the author is muted for retweets, check if this is a retweet
                if tw.rt_tweet.is_some() {
                    return true;
                }
            }
            None => {}
        };

        if let Some(ref rt_id) = tw.rt_tweet {
            if let Some(rt) = self.retrieve_tweet(&TweetId::Twitter(rt_id.to_owned())) {
                match self.mutes.users.get(&rt.author_id) {
                    Some(Everything) => { return true; },
                    Some(Retweet) => { /* Retweets don't show up as retweets of retweets, ever */ },
                    Some(Mentions) => { /* what does this entail? would the rewteet have to mention you? */ },
                    None => {}
                }
            }
        }

        // Check reply_id for mutes independently because the mute
        // may be on a tweet we don't have cached
        if let Some(ref reply_id) = tw.reply_to_tweet {
            match self.mutes.tweets.get(reply_id) {
                Some(Conversation) => {
                    return true;
                }
                // Other mute levels are to mute notifications, like rt/fav, not conversation
                //
                // What about quote_tweet notifications?
                _ => {}
            }

            if let Some(reply) = self.retrieve_tweet(&TweetId::Twitter(reply_id.to_owned())) {
                match self.mutes.users.get(&reply.author_id) {
                    Some(Everything) => {
                        return true;
                    },
                    _ => {}
                }
            }
        }

        // If we have the author, see if the author is muted

        return false;
    }

    fn event_muted(&self, ev: &events::Event) -> bool {
        match ev {
            &events::Event::Deleted { ref user_id, ref twete_id } => {
                if self.mutes.users.contains_key(user_id) || self.mutes.tweets.contains_key(twete_id) {
                    return true;
                }
            },
            &events::Event::RT_RT { ref user_id, ref twete_id } => {
                if self.mutes.users.contains_key(user_id) || self.mutes.tweets.contains_key(twete_id) {
                    return true;
                }
            },
            &events::Event::Fav_RT { ref user_id, ref twete_id } => {
                if self.mutes.users.contains_key(user_id) || self.mutes.tweets.contains_key(twete_id) {
                    return true;
                }
            },
            &events::Event::Fav { ref user_id, ref twete_id } => {
                if self.mutes.users.contains_key(user_id) || self.mutes.tweets.contains_key(twete_id) {
                    return true;
                }
            },
            &events::Event::Unfav { ref user_id, ref twete_id } => {
                if self.mutes.users.contains_key(user_id) || self.mutes.tweets.contains_key(twete_id) {
                    return true;
                }
            },
            &events::Event::Quoted { ref user_id, ref twete_id } => {
                if self.mutes.users.contains_key(user_id) || self.mutes.tweets.contains_key(twete_id) {
                    return true;
                }
            },
            &events::Event::Followed { ref user_id } => {
                if self.mutes.users.contains_key(user_id) {
                    return true;
                }
            },
            &events::Event::Unfollowed { ref user_id } => {
                if self.mutes.users.contains_key(user_id) {
                    return true;
                }
            }
        }

        // TODO: if there's a referenced tweet, see if the tweet is part of a conversation mute

        return false;
    }

    fn new_without_caching() -> TwitterCache {
        let mut cache = TwitterCache::new();
        cache.caching_permitted = false;
        cache
    }
    pub fn add_profile(&mut self, profile: TwitterProfile, name: Option<String>, display_info: &mut DisplayInfo) {
        self.profiles.insert(name.unwrap_or(profile.user.handle.to_owned()), profile);
        if self.caching_permitted {
            self.store_cache(display_info);
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
    pub fn store_cache(&mut self, display_info: &mut DisplayInfo) {
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
                display_info.status("No cache dir exists...".to_owned());
            }
        }
    }
    fn number_and_insert_tweet(&mut self, mut tw: Tweet) {
        if !self.tweets.contains_key(&tw.id.to_owned()) {
            if tw.internal_id == 0 {
                tw.internal_id = (self.tweets.len() as u64) + 1;
                self.id_conversions.id_to_tweet_id.insert(tw.internal_id, tw.id.to_owned());
                let local_recv_time = tw.recieved_at.with_timezone(&Local);
                let tweet_date = format!("{:04}{:02}{:02}", local_recv_time.year(), local_recv_time.month(), local_recv_time.day());
                if !self.id_conversions.tweets_by_date.contains_key(&tweet_date) {
                    self.id_conversions.tweets_by_date.insert(tweet_date.clone(), HashMap::new());
                }
                if !self.id_conversions.tweets_by_date_and_tweet_id.contains_key(&tweet_date) {
                    self.id_conversions.tweets_by_date_and_tweet_id.insert(tweet_date.clone(), HashMap::new());
                }

                let date_map: &mut HashMap<u64, u64> = self.id_conversions.tweets_by_date.get_mut(&tweet_date).unwrap();
                let next_idx = date_map.len() as u64;
                date_map.insert(next_idx, tw.internal_id);

                let date_map: &mut HashMap<u64, u64> = self.id_conversions.tweets_by_date_and_tweet_id.get_mut(&tweet_date).unwrap();
                let next_idx = date_map.len() as u64;
                date_map.insert(tw.internal_id, next_idx);

                self.tweets.insert(tw.id.to_owned(), tw);
            }
        }
    }
    pub fn display_id_for_tweet(&self, tweet: &Tweet) -> TweetId {
        let now = Local::now();
        let tweet_date = tweet.recieved_at.with_timezone(&Local);
        let bare_id = TweetId::Bare(tweet.internal_id);
        let maybe_dated_id = if now.year() == tweet_date.year() && now.month() == tweet_date.month() && now.day() == tweet_date.day() {
            let date_string = format!("{:04}{:02}{:02}", tweet_date.year(), tweet_date.month(), tweet_date.day());
            let today_id = self.id_conversions.tweets_by_date_and_tweet_id.get(&date_string).and_then(|m| m.get(&tweet.internal_id));
            today_id.map(|x| TweetId::Today(*x))
        } else {
            None
        };

        maybe_dated_id.unwrap_or(bare_id)
    }
    pub fn display_id_for_tweet_id(&self, twid: &TweetId) -> TweetId {
        self.id_conversions.to_display_id(twid, self)
    }
    pub fn load_cache(display_info: &mut DisplayInfo) -> TwitterCache {
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
                            display_info.status(format!("Error reading profile, profile caching disabled... {}", e));
                            cache
                        }
                    }
                }
                Err(e) => {
                    let mut cache = TwitterCache::new_without_caching();
                    display_info.status(format!("Error reading cached profile: {}. Profile caching disabled.", e));
                    cache
                }
            }
        } else {
            let mut cache = TwitterCache::new();
            display_info.status(format!("Hello! First time setup?"));
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
    pub fn cache_api_event(&mut self, conn_id: u8, json: serde_json::Map<String, serde_json::Value>, mut queryer: &mut ::Queryer, display_info: &mut DisplayInfo) {
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
                self.fetch_user(&user_id, &mut queryer, display_info);
            },
            Some("follow") => {
                let follower = json["source"]["id_str"].as_str().unwrap().to_string();
                let followed = json["target"]["id_str"].as_str().unwrap().to_string();
                self.cache_api_user(json["target"].clone());
                self.cache_api_user(json["source"].clone());
                let profile = self.mut_profile_for_connection(conn_id);
                if follower == profile.user.handle {
                    // self.add_follow(
                } else {
                    profile.add_follower(&follower);
                }
            },
            Some("unfollow") => {
                let follower = json["source"]["id_str"].as_str().unwrap().to_string();
                let followed = json["target"]["id_str"].as_str().unwrap().to_string();
                self.cache_api_user(json["target"].clone());
                self.cache_api_user(json["source"].clone());
                let profile = self.mut_profile_for_connection(conn_id);
                if follower == profile.user.handle {
                    // self.add_follow(
                } else {
                    profile.add_follower(&follower);
                }
            },
            Some(_) => () /* an uninteresting event */,
            None => () // not really an event? should we log something?
            /* nothing else to care about now, i think? */
        }
    }
    pub fn retrieve_tweet(&self, tweet_id: &TweetId) -> Option<&Tweet> {
        let maybe_tweet_id = self.id_conversions.to_twitter_id(tweet_id.to_owned());
        maybe_tweet_id.and_then(|id| self.tweets.get(&id))
    }
    pub fn retrieve_user(&self, user_id: &String) -> Option<&User> {
        self.users.get(user_id)
    }
    pub fn fetch_tweet(&mut self, tweet_id: &TweetId, mut queryer: &mut ::Queryer, display_info: &mut DisplayInfo) -> Option<&Tweet> {
        if let &TweetId::Twitter(ref id) = tweet_id {
            if !self.tweets.contains_key(id) {
                match self.look_up_tweet(id, &mut queryer) {
                    Ok(json) => self.cache_api_tweet(json),
                    Err(e) => display_info.status(format!("Unable to retrieve tweet {}:\n{}", id, e))
                };
            }
        }
        self.retrieve_tweet(tweet_id)
    }
    pub fn fetch_user(&mut self, user_id: &String, mut queryer: &mut ::Queryer, display_info: &mut DisplayInfo) -> Option<&User> {
        if !self.users.contains_key(user_id) {
            let maybe_parsed = self.look_up_user(user_id, &mut queryer).and_then(|x| User::from_json(x));
            match maybe_parsed {
                Ok(tw) => self.cache_user(tw),
                Err(e) => display_info.status(format!("Unable to retrieve user {}:\n{}", user_id, e))
            }
        }
        self.users.get(user_id)
    }

    fn look_up_user(&mut self, id: &str, queryer: &mut ::Queryer) -> Result<serde_json::Value, String> {
        match self.current_profile() {
            Some(ref user_profile) => queryer.do_api_get(::USER_LOOKUP_URL, &vec![("user_id", id)], &self.app_key, &user_profile.creds),
            None => Err("No authorized user to conduct lookup".to_owned())
        }
    }

    fn look_up_tweet(&mut self, id: &str, queryer: &mut ::Queryer) -> Result<serde_json::Value, String> {
        match self.current_profile() {
            Some(ref user_profile) => queryer.do_api_get(::TWEET_LOOKUP_URL, &vec![("id", id)], &self.app_key, &user_profile.creds),
            None => Err("No authorized user to conduct lookup".to_owned())
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
    conn_id: u8,
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    display_info: &mut DisplayInfo,
    queryer: &mut ::Queryer) {
    tweeter.cache_api_event(conn_id, structure.clone(), queryer, display_info);
    match events::Event::from_json(structure) {
        Ok(event) => {
            if !tweeter.event_muted(&event) {
                display_info.recv(display::Infos::Event(event));
            }
        },
        Err(e) => {
            display_info.status(format!("Unknown twitter json: {:?}", e));
        }
    }
}

fn handle_twitter_delete(
    conn_id: u8,
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    display_info: &mut DisplayInfo,
    _queryer: &mut ::Queryer) {
    /*
    display_info.recv(display::Infos::Event(
        events::Event::Deleted {
            user_id: structure["delete"]["status"]["user_id_str"].as_str().unwrap().to_string(),
            twete_id: structure["delete"]["status"]["id_str"].as_str().unwrap().to_string()
        }
    ));
    */
}

fn handle_twitter_twete(
    conn_id: u8,
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    display_info: &mut DisplayInfo,
    _queryer: &mut ::Queryer) {
    //display_info.recv(display::Infos::Text(vec![format!("{:?}", structure)]));
    let twete_id = TweetId::Twitter(
        structure["id_str"].as_str().unwrap().to_string()
    );
    tweeter.cache_api_tweet(serde_json::Value::Object(structure));
    if let Some(twete) = tweeter.retrieve_tweet(&twete_id) {
        if !tweeter.tweet_muted(twete) {
            display_info.recv(display::Infos::Tweet(twete_id));
        }
    }
}

fn handle_twitter_dm(
    conn_id: u8,
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    display_info: &mut DisplayInfo,
    _queryer: &mut ::Queryer) {
    // show DM
    tweeter.cache_api_user(structure["direct_message"]["recipient"].clone());
    tweeter.cache_api_user(structure["direct_message"]["sender"].clone());
    let dm_text = structure["direct_message"]["text"].as_str().unwrap().to_string()
        .replace("&amp;", "&")
        .replace("&gt;", ">")
        .replace("&lt;", "<");
    let to = structure["direct_message"]["recipient_id_str"].as_str().unwrap().to_string();
    let from = structure["direct_message"]["sender_id_str"].as_str().unwrap().to_string();
    display_info.recv(display::Infos::DM(dm_text, from, to));
}

fn handle_twitter_welcome(
    conn_id: u8,
    structure: serde_json::Map<String, serde_json::Value>,
    tweeter: &mut TwitterCache,
    display_info: &mut DisplayInfo,
    queryer: &mut ::Queryer) {
    let app_key = tweeter.app_key.clone();
    let followers_changes = {
        let mut profile = tweeter.mut_profile_for_connection(conn_id);

        let settings = profile.get_settings(queryer, &app_key).unwrap();

        let user_id_nums = structure["friends"].as_array().unwrap();
        let user_id_strs = user_id_nums.into_iter().map(|x| x.as_u64().unwrap().to_string());
        let (new_following, lost_following) = profile.set_following(user_id_strs.collect());

        let maybe_my_name = settings["screen_name"].as_str();

        profile.get_followers(queryer, &app_key).map(|followers| {
            let id_arr: Vec<String> = followers["ids"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_owned()).collect();
            (maybe_my_name.unwrap().to_owned(), new_following, lost_following, profile.set_followers(id_arr))
        })
    };

    match followers_changes {
        Ok((my_name, new_following, lost_following, (new_followers, lost_followers))) => {
            /*
             * This *will* spam you on login, and isn't very useful.
             * TODO: present this sanely.
             *
            for user in new_following {
                display_info.status(format!("New following! {}", user));
            }
            for user in lost_following {
                display_info.status(format!("Not following {} anymore", user));
            }
            for user in new_followers {
                display_info.status(format!("New follower! {}", user));
            }
            for user in lost_followers {
                display_info.status(format!("{} isn't following anymore", user));
            }
             */
        },
        Err(e) => {
            display_info.status(e);
        }
    }
}

pub fn handle_message(
    conn_id: u8,
    twete: serde_json::Value,
    tweeter: &mut TwitterCache,
    display_info: &mut DisplayInfo,
    queryer: &mut ::Queryer
) {
    match twete {
        serde_json::Value::Object(objmap) => {
            if objmap.contains_key("event") {
                handle_twitter_event(conn_id, objmap, tweeter, display_info, queryer);
            } else if objmap.contains_key("friends") {
                handle_twitter_welcome(conn_id, objmap, tweeter, display_info, queryer);
            } else if objmap.contains_key("delete") {
                handle_twitter_delete(conn_id, objmap, tweeter, display_info, queryer);
            } else if objmap.contains_key("user") && objmap.contains_key("id") {
                handle_twitter_twete(conn_id, objmap, tweeter, display_info, queryer);
            } else if objmap.contains_key("direct_message") {
                handle_twitter_dm(conn_id, objmap, tweeter, display_info, queryer);
            } else {
                display_info.status(format!("Unknown json: {:?}", objmap));
            }
//            self.display_info.status("");
        },
        _ => ()
    };
}
