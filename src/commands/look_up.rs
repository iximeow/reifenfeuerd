use tw;
use tw::TweetId;
use display;
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
    match TweetId::parse(line) {
        Ok(twid) => {
            if let Some(tweet) = tweeter.fetch_tweet(&twid, &mut queryer).map(|x| x.clone()) {
                tweeter.display_info.recv(display::Infos::Tweet(twid));
            } else {
                tweeter.display_info.status(format!("Couldn't retrieve {:?}", twid));
            }
        },
        Err(e) => {
            tweeter.display_info.status(format!("Invalid id {:?}", e));
        }
    }
}
