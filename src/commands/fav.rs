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
    // TODO handle this unwrap
//    let inner_twid = u64::from_str(&line).unwrap();
    let maybe_id = TweetId::parse(line.to_owned());
    match maybe_id {
        Ok(twid) => {
            let twete = tweeter.retrieve_tweet(&twid).unwrap();
            queryer.do_api_post(&format!("{}?id={}", UNFAV_TWEET_URL, twete.id));
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
    // TODO handle this unwrap
    let maybe_id = TweetId::parse(line.to_owned());
    match maybe_id {
        Ok(twid) => {
            let twete = tweeter.retrieve_tweet(&twid).unwrap();
            queryer.do_api_post(&format!("{}?id={}", FAV_TWEET_URL, twete.id));
        }
        Err(e) => {
            println!("Invalid id: {}", e);
        }
    }
}
