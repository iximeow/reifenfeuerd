use tw;
use ::Queryer;

use commands::Command;

pub static SHOW_CACHE: Command = Command {
    keyword: "show_cache",
    params: 0,
    exec: show_cache
};

fn show_cache(line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer) {
    println!("----* USERS *----");
    for (uid, user) in &tweeter.users {
        println!("User: {} -> {:?}", uid, user);
    }
    println!("----* TWEETS *----");
    for (tid, tweet) in &tweeter.tweets {
        println!("Tweet: {} -> {:?}", tid, tweet);
    }
    println!("----* FOLLOWERS *----");
    for uid in &tweeter.followers.clone() {
        let user_res = tweeter.fetch_user(uid, &mut queryer);
        match user_res {
            Some(user) => {
                println!("Follower: {} - {:?}", uid, user);
            }
            None => { println!("  ..."); }
        }
    }
}
