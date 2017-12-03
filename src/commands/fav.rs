use display::DisplayInfo;
use tw;
use ::Queryer;

use tw::TweetId;

use commands::Command;

static FAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/create.json";
static UNFAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/destroy.json";

pub static UNFAV: Command = Command {
    keyword: "unfav",
    params: 1,
    exec: unfav,
    param_str: " <tweet_id>",
    help_str: "Unfavorite a tweet."
};

fn unfav(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let maybe_id = TweetId::parse(line.to_owned());
    match maybe_id {
        Ok(twid) => {
            if let Some(twete) = tweeter.retrieve_tweet(&twid) {
                let result = match tweeter.current_profile() {
                    Some(user_profile) => queryer.do_api_post(&format!("{}?id={}", UNFAV_TWEET_URL, twete.id), &tweeter.app_key, &user_profile.creds),
                    None => Err("No logged in user to unfav from".to_owned())
                };
                match result {
                    Ok(_) => (),
                    Err(e) => display_info.status(e)
                }
            } else {
                display_info.status(format!("No tweet for id: {:?}", twid));
            }
        }
        Err(e) => {
            display_info.status(format!("Invalid id: {}", e));
        }
    }
}

pub static FAV: Command = Command {
    keyword: "fav",
    params: 1,
    exec: fav,
    param_str: " <tweet_id>",
    help_str: "Favorite a tweet."
};

fn fav(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let maybe_id = TweetId::parse(line.to_owned());
    match maybe_id {
        Ok(twid) => {
            // tweeter.to_twitter_tweet_id(twid)...
            if let Some(twete) = tweeter.retrieve_tweet(&twid) {
                let result = match tweeter.current_profile() {
                    Some(user_profile) => queryer.do_api_post(&format!("{}?id={}", FAV_TWEET_URL, twete.id), &tweeter.app_key, &user_profile.creds),
                    None => Err("No logged in user to fav from".to_owned())
                };
                match result {
                    Ok(_) => (),
                    Err(e) => display_info.status(e)
                }
            } else {
                display_info.status(format!("No tweet for id: {:?}", twid));
            }
        }
        Err(e) => {
            display_info.status(format!("Invalid id: {}", e));
        }
    }
}
