use tw;
use ::Queryer;

use tw::TweetId;

use commands::Command;

static FAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/create.json";
static UNFAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/destroy.json";

pub static UNFAV: Command = Command {
    keyword: "unfav",
    params: 1,
    exec: unfav
};

fn unfav(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let maybe_id = TweetId::parse(line.to_owned());
    match maybe_id {
        Ok(twid) => {
            if let Some(twete) = tweeter.retrieve_tweet(&twid).map(|x| x.clone()) { // TODO: no clone when this stops taking &mut self
                queryer.do_api_post(&format!("{}?id={}", UNFAV_TWEET_URL, twete.id));
            } else {
                tweeter.display_info.status(format!("No tweet for id: {:?}", twid));
            }
        }
        Err(e) => {
            println!("Invalid id: {}", e);
        }
    }
}

pub static FAV: Command = Command {
    keyword: "fav",
    params: 1,
    exec: fav
};

fn fav(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let maybe_id = TweetId::parse(line.to_owned());
    match maybe_id {
        Ok(twid) => {
            // tweeter.to_twitter_tweet_id(twid)...
            if let Some(twete) = tweeter.retrieve_tweet(&twid).map(|x| x.clone()) { // TODO: no clone when this stops taking &mut self
                queryer.do_api_post(&format!("{}?id={}", FAV_TWEET_URL, twete.id));
            } else {
                tweeter.display_info.status(format!("No tweet for id: {:?}", twid));
            }
        }
        Err(e) => {
            println!("Invalid id: {}", e);
        }
    }
}
