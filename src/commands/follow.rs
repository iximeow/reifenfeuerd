use tw;
use ::Queryer;

use commands::Command;

static FOLLOW_URL: &str = "https://api.twitter.com/1.1/friendships/create.json";
static UNFOLLOW_URL: &str = "https://api.twitter.com/1.1/friendships/destroy.json";

pub static UNFOLLOW: Command = Command {
    keyword: "unfl",
    params: 1,
    exec: unfl
};

fn unfl(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let screen_name = line.trim();
    match queryer.do_api_post(&format!("{}?screen_name={}", FOLLOW_URL, screen_name)) {
        Ok(_resp) => (),
        Err(e) => tweeter.display_info.status(format!("unfl request error: {}", e))
    }
}

pub static FOLLOW: Command = Command {
    keyword: "fl",
    params: 1,
    exec: fl
};

fn fl(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let screen_name = line.trim();
    tweeter.display_info.status(format!("fl resp: {:?}", queryer.do_api_post(&format!("{}?screen_name={}", UNFOLLOW_URL, screen_name))));
}
