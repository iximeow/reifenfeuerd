use display::DisplayInfo;
use serde_json;
use tw;
use ::Queryer;

use tw::TweetId;

use commands::Command;

static DEL_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/destroy";
static RT_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/retweet";
static CREATE_TWEET_URL: &str = "https://api.twitter.com/1.1/statuses/update.json";

pub static DEL: Command = Command {
    keyword: "del",
    params: 1,
    exec: del,
    param_str: " <tweet_id>",
    help_str: "Delete tweet <tweet_id>"
};

fn del(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    match TweetId::parse(line.clone()) {
        Ok(twid) => {
            // TODO this really converts twid to a TweetId::Twitter
            if let Some(twitter_id) = tweeter.retrieve_tweet(&twid).map(|x| x.id.to_owned()) {
                let result = match tweeter.current_profile() {
                    Some(user_profile) => queryer.do_api_post_noparam(&format!("{}/{}.json", DEL_TWEET_URL, twitter_id), &tweeter.app_key, &user_profile.creds),
                    None => Err("No logged in user to delete as".to_owned())
                };
                match result {
                    Ok(_) => (),
                    Err(e) => display_info.status(e)
                }
            } else {
                display_info.status(format!("No tweet for id {:?}", twid));
            }
        },
        Err(e) => {
            display_info.status(format!("Invalid id: {:?} ({})", line, e));
        }
    }
}

pub static TWETE: Command = Command {
    keyword: "t",
    params: 0,
    exec: twete,
    param_str: "",
    help_str: "Enter tweet compose mode."
};

fn twete(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    // if there's text, send it.
    // if it's just "t", enter compose mode.
    let text = line.trim().to_owned();
    if text.len() == 0 {
        display_info.set_mode(Some(::display::DisplayMode::Compose(text)));
    } else {
        send_twete(text, tweeter, queryer, display_info);
    }
}

pub fn send_twete(text: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let result = make_tweet(&text, vec![("status", &text)], queryer, tweeter);
    match result {
        Ok(_) => (),
        Err(e) => display_info.status(e)
    }
}

pub static THREAD: Command = Command {
    keyword: "thread",
    params: 2,
    exec: thread,
    param_str: " <tweet_id> <response>", // should be optional..
    // TODO: make it actually do this..
    help_str: "Enter compose mode, appending to a thread"
};

// the difference between threading and replying is not including
// yourself in th @'s.
fn thread(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let user_profile = match tweeter.current_profile().map(|profile| profile.to_owned()) {
        Some(profile) => profile,
        None => {
            display_info.status("To reply you must be authenticated as a user.".to_owned());
            return;
        }
    };
    let mut text: String = line.trim().to_string();
    if let Some(id_end_idx) = text.find(" ") {
        let reply_bare = text.split_off(id_end_idx + 1);
        let reply = reply_bare.trim();
        let id_str = text.trim();
        if reply.len() > 0 {
            let maybe_id = TweetId::parse(id_str.to_owned());
            match maybe_id {
                Ok(twid) => {
                    if let Some(twete) = tweeter.retrieve_tweet(&twid).map(|x| x.clone()) { // TODO: no clone when this stops taking &mut self
                        let handle = &tweeter.retrieve_user(&twete.author_id).unwrap().handle.to_owned();
                        // TODO: definitely breaks if you change your handle right now
                        if handle == &user_profile.user.handle {
                            send_reply(reply.to_owned(), twid, tweeter, queryer, user_profile.creds, display_info);
                        } else {
                            display_info.status("you can only thread your own tweets".to_owned());
                            // ask if it should .@ instead?
                        }
                    }
                }
                Err(e) => {
                    display_info.status(format!("Invalid id: {}", e));
                }
            }
        } else {
            display_info.status("thread <id> your sik reply".to_owned());
        }
    } else {
        display_info.status("thread <id> your sik reply".to_owned());
    }
}

pub static REP: Command = Command {
    keyword: "rep",
    params: 1,
    exec: rep,
    param_str: " <tweet_id>",
    // TODO: doc immediate reply mode
    help_str: "Enter compose mode to reply to <tweet_id>"
};

fn rep(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let user_profile = match tweeter.current_profile().map(|profile| profile.to_owned()) {
        Some(profile) => profile,
        None => {
            display_info.status("To reply you must be authenticated as a user.".to_owned());
            return;
        }
    };
    let mut text: String = line.trim().to_string();
    let reply_bare = match text.find(" ") {
        None => "".to_owned(),
        Some(id_end_idx) => {
            text.split_off(id_end_idx + 1)
        }
    };
    let reply = reply_bare.trim();
    let id_str = text.trim();
    let maybe_id = TweetId::parse(id_str.to_owned());
    match maybe_id {
        Ok(twid) => {
            if let Some(twete) = tweeter.retrieve_tweet(&twid).map(|x| x.clone()) { // TODO: no clone when this stops taking &mut self
                // get handles to reply to...
                let author_handle = tweeter.retrieve_user(&twete.author_id).unwrap().handle.to_owned();
                let raw_ats: Vec<String> = twete.get_mentions(); //std::collections::HashSet::new();
                let mut reordered_ats: Vec<String> = Vec::new();
                if &author_handle != &user_profile.user.handle {
                    reordered_ats.push(author_handle);
                }
                for handle in raw_ats.into_iter() {
                    if &handle != &user_profile.user.handle {
                        reordered_ats.push(handle);
                    }
                }
                if let Some(rt_tweet) = twete.rt_tweet.and_then(|id| tweeter.retrieve_tweet(&TweetId::Twitter(id))).map(|x| x.clone()) {
                    let rt_author_handle = tweeter.retrieve_user(&rt_tweet.author_id).unwrap().handle.to_owned();
                    let mut reordered_with_rt = Vec::new();
                    for mention in reordered_ats {
                        if mention != rt_author_handle {
                            reordered_with_rt.push(mention);
                        }
                    }
                    reordered_with_rt.insert(1, rt_author_handle);
                    reordered_ats = reordered_with_rt;
                }

                //let ats_vec: Vec<&str> = ats.into_iter().collect();
                //let full_reply = format!("{} {}", ats_vec.join(" "), reply);
                let decorated_ats: Vec<String> = reordered_ats.into_iter().map(|x| format!("@{}", x)).collect();
                let full_reply = format!("{} {}", decorated_ats.join(" "), reply);

                if reply.len() > 0 {
                    send_reply(full_reply, twid, tweeter, queryer, user_profile.creds, display_info);
                } else {
                    display_info.set_mode(Some(::display::DisplayMode::Reply(twid, full_reply)));
                }
            } else {
                display_info.status(format!("No tweet for id: {:?}", twid));
            }
        },
        Err(e) => {
            display_info.status(format!("Cannot parse input: {:?} ({})", id_str, e));
        }
    }
}

pub fn make_tweet(text: &str, params: Vec<(&str, &str)>, queryer: &mut Queryer, tweeter: &tw::TwitterCache) -> Result<serde_json::Value, String> {
    let updated_text = if tweeter.translate_emoji {
        text
            .replace(":thinking:", "🤔")
            .replace(":clap:", "👏")
    } else {
        text.to_owned()
    };
    let mut all_params = params;
    all_params.push(("status", &updated_text));

    match tweeter.current_profile() {
        Some(user_profile) => queryer.do_api_post(CREATE_TWEET_URL, &all_params, &tweeter.app_key, &user_profile.creds),
        None => Err("No logged in user to tweet as".to_owned())
    }
}

pub fn send_reply(text: String, twid: TweetId, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, user_creds: tw::Credential, display_info: &mut DisplayInfo) {
    if let Some(twete) = tweeter.retrieve_tweet(&twid) {
        let result = make_tweet(&text, vec![("in_reply_to_status_id", &twete.id)], queryer, tweeter);
        match result {
            Ok(_) => (),
            Err(e) => display_info.status(e)
        }
    } else {
        display_info.status(format!("Tweet stopped existing: {:?}", twid));
    }
}

pub static QUOTE: Command = Command {
    keyword: "qt",
    params: 2,
    exec: quote,
    param_str: " <tweet_id> <text>",
    help_str: "Quote <tweet_id> with context <text>"
};

fn quote(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let mut text: String = line.trim().to_string();
    if let Some(id_end_idx) = text.find(" ") {
        let reply_bare = text.split_off(id_end_idx + 1);
        let reply = reply_bare.trim();
        let id_str = text.trim();
        if reply.len() > 0 {
            let maybe_id = TweetId::parse(id_str.to_owned());
            match maybe_id {
                Ok(twid) => {
                    if let Some(twete) = tweeter.retrieve_tweet(&twid) {
                        let attachment_url =
                            &format!(
                                "https://www.twitter.com/{}/status/{}",
                                tweeter.retrieve_user(&twete.author_id).unwrap().handle, // TODO: for now this is ok ish, if we got the tweet we have the author
                                twete.id
                            );
                        let result = make_tweet(reply, vec![("attachment_url", attachment_url)], queryer, tweeter);
                        match result {
                            Ok(_) => (),
                            Err(e) => display_info.status(e)
                        }
                    } else {
                        display_info.status(format!("No tweet found for id {:?}", twid));
                    }
                },
                Err(e) => {
                    display_info.status(format!("Invalid id: {:?}", id_str));
                }
            }
        } else {
            display_info.status("rep <id> your sik reply".to_owned());
        }
    } else {
        display_info.status("rep <id> your sik reply".to_owned());
    }
}

pub static RETWETE: Command = Command {
    keyword: "rt",
    params: 1,
    exec: retwete,
    param_str: " <tweet_id>",
    help_str: "Retweet <tweet_id>"
};

fn retwete(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    match TweetId::parse(line.clone()) {
        Ok(twid) => {
            // TODO this really converts twid to a TweetId::Twitter
            if let Some(twitter_id) = tweeter.retrieve_tweet(&twid).map(|x| x.id.to_owned()) {
                let result = match tweeter.current_profile() {
                    Some(user_profile) => {
                        queryer.do_api_post_noparam(&format!("{}/{}.json", RT_TWEET_URL, twitter_id), &tweeter.app_key, &user_profile.creds)
                    },
                    None => Err("No logged in user to retweet as".to_owned())
                };
                match result {
                    Ok(_) => (),
                    Err(e) => display_info.status(e)
                }
            } else {
                display_info.status(format!("No tweet for id {:?}", twid));
            }
        },
        Err(e) => {
            display_info.status(format!("Invalid id: {:?}", line));
        }
    }
}

