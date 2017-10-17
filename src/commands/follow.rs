use tw;
use ::Queryer;

use commands::Command;

use std::str::FromStr;

static FOLLOW_URL: &str = "https://api.twitter.com/1.1/friendships/create.json";
static UNFOLLOW_URL: &str = "https://api.twitter.com/1.1/friendships/destroy.json";

pub static UNFOLLOW: Command = Command {
    keyword: "unfl",
    params: 1,
    exec: unfl
};

fn unfl(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    // TODO handle this unwrap
    let screen_name = line.trim(); //u64::from_str(&line).unwrap();
    queryer.do_api_post(&format!("{}?screen_name={}", FOLLOW_URL, screen_name));
}

pub static FOLLOW: Command = Command {
    keyword: "fl",
    params: 1,
    exec: fl
};

fn fl(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    // TODO handle this unwrap
    let screen_name = line.trim(); //u64::from_str(&line).unwrap();
    println!("fl resp: {:?}", queryer.do_api_post(&format!("{}?screen_name={}", UNFOLLOW_URL, screen_name)));
}
