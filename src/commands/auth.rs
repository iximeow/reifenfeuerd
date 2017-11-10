use tw;
use std;
use std::collections::HashMap;
use hyper;
use ::Queryer;

use tw::TweetId;

use commands::Command;

static FAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/create.json";
static UNFAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/destroy.json";

pub static AUTH: Command = Command {
    keyword: "auth",
    params: 0,
    exec: auth
};

static OAUTH_REQUEST_TOKEN_URL: &str = "https://api.twitter.com/oauth/request_token";
static OAUTH_AUTHORIZE_URL: &str = "https://api.twitter.com/oauth/authorize";
static OAUTH_ACCESS_TOKEN_URL: &str = "https://api.twitter.com/oauth/access_token";

fn auth(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    // step 0: get an oauth token.
    // https://developer.twitter.com/en/docs/basics/authentication/api-reference/request_token with
    // callback set to oob so the user will later get a PIN.
    // step 1: now present the correect oauth/authorize URL
    // this is as far as auth can get (rest depends on user PIN'ing with the right thing)
    let res = queryer.raw_issue_request(::signed_api_req(&format!("{}?oauth_callback=oob", OAUTH_REQUEST_TOKEN_URL), hyper::Method::Post, &tweeter.app_key));
    match res {
        Ok(bytes) =>
            match std::str::from_utf8(&bytes) {
                Ok(url) => {
                    let parts: Vec<Vec<&str>> = url.split("&").map(|part| part.split("=").collect()).collect();
                    let mut as_map: HashMap<&str, &str> = HashMap::new();
                    for part in parts {
                        as_map.insert(part[0], part[1]);
                    }
                    tweeter.WIP_auth = Some(tw::Credential {
                        key: as_map["oauth_token"].to_owned(),
                        secret: as_map["oauth_token_secret"].to_owned()
                    });
                    tweeter.display_info.status(format!("Now enter `pin` with the code at {}?oauth_token={}", OAUTH_AUTHORIZE_URL, as_map["oauth_token"]));
                }
                Err(_) =>
                    tweeter.display_info.status("couldn't rebuild url".to_owned())
            },
        Err(e) =>
            tweeter.display_info.status(format!("request token url error: {}", e))
    };
}

pub static PIN: Command = Command {
    keyword: "pin",
    params: 1,
    exec: pin
};

fn pin(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    if tweeter.WIP_auth.is_none() {
        tweeter.display_info.status("Begin authorizing an account with `auth` first.".to_owned());
        return;
    }

    let res = queryer.raw_issue_request(::signed_api_req_with_token(&format!("{}?oauth_verifier={}", OAUTH_ACCESS_TOKEN_URL, line), hyper::Method::Post, &tweeter.app_key, &tweeter.WIP_auth.clone().unwrap()));
    match res {
        Ok(bytes) =>
            match std::str::from_utf8(&bytes) {
                Ok(url) => {
                    let parts: Vec<Vec<&str>> = url.split("&").map(|part| part.split("=").collect()).collect();
                    let mut as_map: HashMap<&str, &str> = HashMap::new();
                    for part in parts {
                        as_map.insert(part[0], part[1]);
                    }
                    // turns out the "actual" oauth creds are different
                    tweeter.add_profile(tw::Credential {
                        key: as_map["oauth_token"].to_owned(),
                        secret: as_map["oauth_token_secret"].to_owned()
                    });
                    tweeter.WIP_auth = None;
                    tweeter.state = tw::AppState::Reconnect;
                    tweeter.display_info.status("Looks like you authed! Connecting...".to_owned());
                },
                Err(_) =>
                    tweeter.display_info.status("couldn't rebuild url".to_owned())
            },
        Err(e) =>
            tweeter.display_info.status(format!("request token url error: {}", e))
    };
}
