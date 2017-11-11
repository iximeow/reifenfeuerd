use tw;
use ::Queryer;

use tw::TweetId;

use commands::Command;

use display;

pub static VIEW: Command = Command {
    keyword: "view",
    params: 1,
    exec: view,
    help_str: "<tweet_id>: Display tweet <tweet_id> with a reference URL"
};

fn view(line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    match TweetId::parse(line) {
        Ok(twid) => {
            if let Some(twete) = tweeter.retrieve_tweet(&twid).map(|x| x.clone()) {
                tweeter.display_info.recv(display::Infos::TweetWithContext(
                    TweetId::Twitter(twete.id.to_owned()),
                    format!("link: https://twitter.com/i/web/status/{}", twete.id)
                ));
            } else {
                tweeter.display_info.status(format!("No tweet for id {:?}", twid));
            }
        },
        Err(e) => {
            tweeter.display_info.status(format!("Invalid id {:?}", e));
        }
    }
}

pub static VIEW_THREAD: Command = Command {
    keyword: "view_tr",
    params: 1,
    exec: view_tr,
    help_str: "<tweet_id>: Display whole thread leading to <tweet_id>, reference URLs for each"
};

fn view_tr(line: String, mut tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let mut thread: Vec<TweetId> = Vec::new();
    let maybe_curr_id = TweetId::parse(line);
    match maybe_curr_id {
        Ok(curr_id) => {
            let first_twete = tweeter.fetch_tweet(&curr_id, queryer).map(|x| x.to_owned());
            if first_twete.is_some() {
                thread.push(curr_id);
            }
            let mut maybe_next_id = first_twete.and_then(|x| x.reply_to_tweet.to_owned());
            while let Some(next_id) = maybe_next_id {
                let curr_id = TweetId::Twitter(next_id);
                maybe_next_id = tweeter.fetch_tweet(&curr_id, queryer).and_then(|x| x.reply_to_tweet.to_owned());
                thread.push(curr_id);
            }
        },
        Err(e) => {
            tweeter.display_info.status(format!("Invalid id {:?}", e));
        }
    }

    tweeter.display_info.recv(display::Infos::Thread(thread));
}

pub static VIEW_THREAD_FORWARD: Command = Command {
    keyword: "viewthread+",
    params: 1,
    exec: view_tr_forward,
    help_str: "help me make this work"
};

fn view_tr_forward(_line: String, _tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    // first see if we have a thread for the tweet named
    // if we do not, we'll have to mimic a request like 
    // curl 'https://twitter.com/jojonila/status/914383908090691584' \
    //   -H 'accept-encoding: gzip, deflate, br' \
    //   -H 'user-agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
    //     (KHTML, like Gecko) Chrome/61.0.3163.91 Safari/537.36' \
    //   -H 'x-overlay-request: true'
    /*
     * above results in a response:
     * {
     *  "init_data": { ... },
     *  "title": "tweet, but for page title",
     *  ..
     *  "page": "HTML THAT'S JUST DROPPED IN AS A MODAL."
     *  */
}
