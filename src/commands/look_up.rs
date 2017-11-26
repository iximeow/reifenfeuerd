use display::DisplayInfo;
use tw;
use tw::TweetId;
use display;
use ::Queryer;

use commands::Command;

pub static LOOK_UP_USER: Command = Command {
    keyword: "look_up_user",
    params: 1,
    exec: look_up_user,
    param_str: " <twitter_user_id>",
    help_str: "Look up the user by the specified twitter user ID, display name/handle."
};

fn look_up_user(line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    // should probably just pass the id?
    if let Some(user) = tweeter.fetch_user(&line, &mut queryer, display_info).map(|x| x.clone()) {
        display_info.recv(display::Infos::User(user));
    } else {
        display_info.status(format!("Couldn't retrieve {}", line));
    }
}

pub static LOOK_UP_TWEET: Command = Command {
    keyword: "look_up_tweet",
    params: 1,
    exec: look_up_tweet,
    param_str: " <tweet_id>",
    help_str: "Look up tweet by the tweet ID. If unknown, try to retrieve it."
};

// TODO: make this parse a proper tweet id
fn look_up_tweet(line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    match TweetId::parse(line) {
        Ok(twid) => {
            if let Some(tweet) = tweeter.fetch_tweet(&twid, &mut queryer, display_info).map(|x| x.clone()) {
                display_info.recv(display::Infos::Tweet(twid));
            } else {
                display_info.status(format!("Couldn't retrieve {:?}", twid));
            }
        },
        Err(e) => {
            display_info.status(format!("Invalid id {:?}", e));
        }
    }
}
