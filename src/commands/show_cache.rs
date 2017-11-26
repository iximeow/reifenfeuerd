use display::DisplayInfo;
use tw;
use ::Queryer;

use commands::Command;

pub static SHOW_CACHE: Command = Command {
    keyword: "show_cache",
    params: 0,
    exec: show_cache,
    param_str: "",
    help_str: "Dump all cached info. Probably a bad idea."
};

fn show_cache(_line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    /*
    display_info.status("----* USERS *----".to_owned());
    for (uid, user) in &tweeter.users {
        display_info.status(format!("User: {} -> {:?}", uid, user));
    }
    display_info.status("----* TWEETS *----".to_owned());
    for (tid, tweet) in &tweeter.tweets {
        display_info.status(format!("Tweet: {} -> {:?}", tid, tweet));
    }
    display_info.status("----* FOLLOWERS *----".to_owned());
    for uid in &tweeter.followers.clone() {
        let user_res = tweeter.fetch_user(uid, &mut queryer).map(|x| x.clone());
        match user_res {
            Some(user) => {
                display_info.status(format!("Follower: {} - {:?}", uid, user));
            }
            None => { display_info.status("  ...".to_owned()); }
        }
    }
    */
}
