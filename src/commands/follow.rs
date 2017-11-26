use display::DisplayInfo;
use tw;
use ::Queryer;

use commands::Command;

static FOLLOW_URL: &str = "https://api.twitter.com/1.1/friendships/create.json";
static UNFOLLOW_URL: &str = "https://api.twitter.com/1.1/friendships/destroy.json";

pub static UNFOLLOW: Command = Command {
    keyword: "unfl",
    params: 1,
    exec: unfl,
    param_str: " <handle>",
    help_str: "Unfollow <handle>. No @ prefix in <handle>!"
};

fn unfl(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let screen_name = line.trim();
    let result = match tweeter.current_profile() {
        Some(user_profile) => {
            queryer.do_api_post(&format!("{}?screen_name={}", FOLLOW_URL, screen_name), &tweeter.app_key, &user_profile.creds)
        },
        None => Err("No logged in user to unfollow from".to_owned())
    };
    match result {
        Ok(_resp) => (),
        Err(e) => display_info.status(format!("unfl request error: {}", e))
    }
}

pub static FOLLOW: Command = Command {
    keyword: "fl",
    params: 1,
    exec: fl,
    param_str: " <handle>",
    help_str: "Follow <handle>. No @ prefix in <handle>!"
};

fn fl(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let screen_name = line.trim();
    match tweeter.current_profile().map(|profile| profile.to_owned()) {
        Some(user_profile) => {
            display_info.status(
                format!(
                    "fl resp: {:?}",
                    queryer.do_api_post(
                        &format!("{}?screen_name={}", UNFOLLOW_URL, screen_name),
                        &tweeter.app_key,
                        &user_profile.creds
                    )
                )
            )
        },
        None => display_info.status("No logged in user to follow from".to_owned())
    };
}
