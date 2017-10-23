use tw;
use ::Queryer;

use tw::TweetId;

use commands::Command;

use std::str::FromStr;

use display;

pub static VIEW: Command = Command {
    keyword: "view",
    params: 1,
    exec: view
};

fn view(line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    // TODO handle this unwrap
    let inner_twid = u64::from_str(&line).unwrap();
    let twete = tweeter.retrieve_tweet(&TweetId::Bare(inner_twid)).unwrap().clone();
    tweeter.display_info.recv(display::Infos::Tweet(TweetId::Twitter(twete.id.to_owned())));
//    display::render_twete(&twete.id, tweeter);
//    println!(" link: https://twitter.com/i/web/status/{}", twete.id);
}

pub static VIEW_THREAD: Command = Command {
    keyword: "view_tr",
    params: 1,
    exec: view_tr
};

fn view_tr(line: String, mut tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let mut thread: Vec<TweetId> = Vec::new();
    let inner_twid = u64::from_str(&line).unwrap();
    let curr_id = TweetId::Bare(inner_twid);
    let mut maybe_next_id = tweeter.retrieve_tweet(&curr_id).and_then(|x| x.reply_to_tweet.to_owned());
    thread.push(curr_id);
    while let Some(next_id) = maybe_next_id {
        let curr_id = TweetId::Twitter(next_id);
        maybe_next_id = tweeter.retrieve_tweet(&curr_id).and_then(|x| x.reply_to_tweet.to_owned());
        thread.push(curr_id);
    }

    tweeter.display_info.recv(display::Infos::Thread(thread));
//    display::render_twete(&twete.id, tweeter);
//    println!("link: https://twitter.com/i/web/status/{}", twete.id);
}

pub static VIEW_THREAD_FORWARD: Command = Command {
    keyword: "viewthread+",
    params: 1,
    exec: view_tr_forward
};

fn view_tr_forward(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
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
