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
    exec: del
};

fn del(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    match TweetId::parse(line.clone()) {
        Ok(twid) => {
            // TODO this really converts twid to a TweetId::Twitter
            if let Some(twitter_id) = tweeter.retrieve_tweet(&twid).map(|x| x.id.to_owned()) {
                match queryer.do_api_post(&format!("{}/{}.json", DEL_TWEET_URL, twitter_id)) {
                    Ok(_) => (),
                    Err(e) => tweeter.display_info.status(e)
                }
            } else {
                tweeter.display_info.status(format!("No tweet for id {:?}", twid));
            }
        },
        Err(e) => {
            tweeter.display_info.status(format!("Invalid id: {:?} ({})", line, e));
        }
    }
}

pub static TWETE: Command = Command {
    keyword: "t",
    params: 1,
    exec: twete
};

fn twete(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let text = line.trim();
    let substituted = ::url_encode(text);
    if text.len() <= 140 {
        match queryer.do_api_post(&format!("{}?status={}", CREATE_TWEET_URL, substituted)) {
            Ok(_) => (),
            Err(e) => tweeter.display_info.status(e)
        }
    } else {
        // TODO: this 140 is maybe sometimes 280.. :)
        // and see if weighted_character_count still does things?
        tweeter.display_info.status(format!("tweet is too long: {}/140 chars", text.len()));
    }
}

pub static THREAD: Command = Command {
    keyword: "thread",
    params: 2,
    exec: thread
};

fn thread(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
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
                        if handle == &tweeter.current_user.handle {
                            let substituted = ::url_encode(reply);
                            match queryer.do_api_post(&format!("{}?status={}&in_reply_to_status_id={}", CREATE_TWEET_URL, substituted, twete.id)) {
                                Ok(_) => (),
                                Err(e) => tweeter.display_info.status(e)
                            }
                        } else {
                            tweeter.display_info.status("you can only thread your own tweets".to_owned());
                            // ask if it should .@ instead?
                        }
                    }
                }
                Err(e) => {
                    tweeter.display_info.status(format!("Invalid id: {}", e));
                }
            }
        } else {
            tweeter.display_info.status("thread <id> your sik reply".to_owned());
        }
    } else {
        tweeter.display_info.status("thread <id> your sik reply".to_owned());
    }
}

pub static REP: Command = Command {
    keyword: "rep",
    params: 2,
    exec: rep
};

fn rep(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
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
                        // get handles to reply to...
                        let author_handle = tweeter.retrieve_user(&twete.author_id).unwrap().handle.to_owned();
                        let mut ats: Vec<String> = twete.get_mentions(); //std::collections::HashSet::new();
                        /*
                        for handle in twete.get_mentions() {
                            ats.insert(handle);
                        }
                        */
                        ats.remove_item(&author_handle);
                        ats.insert(0, author_handle);
                        if let Some(rt_tweet) = twete.rt_tweet.and_then(|id| tweeter.retrieve_tweet(&TweetId::Twitter(id))).map(|x| x.clone()) {
                            let rt_author_handle = tweeter.retrieve_user(&rt_tweet.author_id).unwrap().handle.to_owned();
                            ats.remove_item(&rt_author_handle);
                            ats.insert(1, rt_author_handle);
                        }
                        //let ats_vec: Vec<&str> = ats.into_iter().collect();
                        //let full_reply = format!("{} {}", ats_vec.join(" "), reply);
                        let decorated_ats: Vec<String> = ats.into_iter().map(|x| format!("@{}", x)).collect();
                        let full_reply = format!("{} {}", decorated_ats.join(" "), reply);
                        let substituted = ::url_encode(&full_reply);
                        match queryer.do_api_post(&format!("{}?status={}&in_reply_to_status_id={}", CREATE_TWEET_URL, substituted, twete.id)) {
                            Ok(_) => (),
                            Err(e) => tweeter.display_info.status(e)
                        }
                    } else {
                        tweeter.display_info.status(format!("No tweet for id: {:?}", twid));
                    }
                },
                Err(e) => {
                    tweeter.display_info.status(format!("Cannot parse input: {:?} ({})", id_str, e));
                }
            }
        } else {
            tweeter.display_info.status("rep <id> your sik reply".to_owned());
        }
    } else {
        tweeter.display_info.status("rep <id> your sik reply".to_owned());
    }
}

pub static QUOTE: Command = Command {
    keyword: "qt",
    params: 2,
    exec: quote
};

fn quote(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
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
                        let substituted = ::url_encode(reply);
                        let attachment_url = ::url_encode(
                            &format!(
                                "https://www.twitter.com/{}/status/{}",
                                tweeter.retrieve_user(&twete.author_id).unwrap().handle, // TODO: for now this is ok ish, if we got the tweet we have the author
                                twete.id
                            )
                        );
                        match queryer.do_api_post(
                            &format!("{}?status={}&attachment_url={}",
                                     CREATE_TWEET_URL,
                                     substituted,
                                     attachment_url
                            )
                        ) {
                            Ok(_) => (),
                            Err(e) => tweeter.display_info.status(e)
                        }
                    } else {
                        tweeter.display_info.status(format!("No tweet found for id {:?}", twid));
                    }
                },
                Err(e) => {
                    tweeter.display_info.status(format!("Invalid id: {:?}", id_str));
                }
            }
        } else {
            tweeter.display_info.status("rep <id> your sik reply".to_owned());
        }
    } else {
        tweeter.display_info.status("rep <id> your sik reply".to_owned());
    }
}

pub static RETWETE: Command = Command {
    keyword: "rt",
    params: 1,
    exec: retwete
};

fn retwete(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    match TweetId::parse(line.clone()) {
        Ok(twid) => {
            // TODO this really converts twid to a TweetId::Twitter
            if let Some(twitter_id) = tweeter.retrieve_tweet(&twid).map(|x| x.id.to_owned()) {
                match queryer.do_api_post(&format!("{}/{}.json", RT_TWEET_URL, twitter_id)) {
                    Ok(_) => (),
                    Err(e) => tweeter.display_info.status(e)
                }
            } else {
                tweeter.display_info.status(format!("No tweet for id {:?}", twid));
            }
        },
        Err(e) => {
            tweeter.display_info.status(format!("Invalid id: {:?}", line));
        }
    }
}

