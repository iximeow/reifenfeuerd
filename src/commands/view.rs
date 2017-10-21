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
    let twete = tweeter.retrieve_tweet(&TweetId::Bare(inner_twid)).unwrap();
    display::render_twete(&twete.id, tweeter);
    println!(" link: https://twitter.com/i/web/status/{}", twete.id);
}

pub static VIEW_THREAD: Command = Command {
    keyword: "view_tr",
    params: 1,
    exec: view_tr
};

fn view_tr(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    // TODO handle this unwrap
    let inner_twid = u64::from_str(&line).unwrap();
    view_tr_inner(inner_twid, tweeter, queryer);
}

fn view_tr_inner(id: u64, mut tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let twete: tw::tweet::Tweet = tweeter.retrieve_tweet(&TweetId::Bare(id)).unwrap().to_owned();
    if let Some(reply_id) = twete.reply_to_tweet.clone() {
        if let Some(reply_internal_id) = tweeter.fetch_tweet(&reply_id, queryer).map(|x| x.internal_id).map(|x| x.to_owned()) {
            view_tr_inner(reply_internal_id, tweeter, queryer);
            println!("      |");
            println!("      v");
        }
    }
    display::render_twete(&twete.id, tweeter);
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
