use tw;
use ::Queryer;

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
    let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
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
    let twete: tw::tweet::Tweet = tweeter.tweet_by_innerid(id).unwrap().to_owned();
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
