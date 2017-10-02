use tw;
use ::Queryer;

use commands::Command;

use std::str::FromStr;

pub static VIEW: Command = Command {
    keyword: "view",
    params: 1,
    exec: view
};

fn view(line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    // TODO handle this unwrap
    let inner_twid = u64::from_str(&line).unwrap();
    let twete = tweeter.tweet_by_innerid(inner_twid).unwrap();
    ::render_twete(&twete.id, tweeter);
    println!("link: https://twitter.com/i/web/status/{}", twete.id);
}
