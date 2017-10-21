use tw;
use ::Queryer;

use commands::Command;

pub static LOOK_UP_USER: Command = Command {
    keyword: "look_up_user",
    params: 1,
    exec: look_up_user
};

fn look_up_user(line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer) {
    if let Some(user) = tweeter.fetch_user(&line, &mut queryer) {
        println!("{:?}", user);
    } else {
//            println!("Couldn't retrieve {}", userid);
    }
}

pub static LOOK_UP_TWEET: Command = Command {
    keyword: "look_up_tweet",
    params: 1,
    exec: look_up_tweet
};

// TODO: make this parse a proper tweet id
fn look_up_tweet(line: String, tweeter: &mut tw::TwitterCache, mut queryer: &mut Queryer) {
    if let Some(tweet) = tweeter.fetch_tweet(&line, &mut queryer) {
        println!("{:?}", tweet);
    } else {
//            println!("Couldn't retrieve {}", tweetid);
    }
}
