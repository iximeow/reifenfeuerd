use display::DisplayInfo;
use tw;
use std;
use std::collections::HashMap;
use hyper;
use ::Queryer;

use commands::Command;

static FAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/create.json";
static UNFAV_TWEET_URL: &str = "https://api.twitter.com/1.1/favorites/destroy.json";

pub static AUTH: Command = Command {
    keyword: "auth",
    params: 0,
    exec: auth,
    param_str: "",
    // TODO: support account-specific auth? profile name spec?
    help_str: "Begin PIN-based account auth process. Second step is the `pin` command."
};

static OAUTH_REQUEST_TOKEN_URL: &str = "https://api.twitter.com/oauth/request_token";
static OAUTH_AUTHORIZE_URL: &str = "https://api.twitter.com/oauth/authorize";
static OAUTH_ACCESS_TOKEN_URL: &str = "https://api.twitter.com/oauth/access_token";

fn auth(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
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
                    display_info.status(format!("Now enter `pin` with the code at {}?oauth_token={}", OAUTH_AUTHORIZE_URL, as_map["oauth_token"]));
                }
                Err(_) =>
                    display_info.status("couldn't rebuild url".to_owned())
            },
        Err(e) =>
            display_info.status(format!("request token url error: {}", e))
    };
}

pub static PIN: Command = Command {
    keyword: "pin",
    params: 1,
    exec: pin,
    param_str: " <PIN>",
    help_str: "Complete account auth. Enter PIN from prior `auth` link to connect an account."
};

fn pin(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    if tweeter.WIP_auth.is_none() {
        display_info.status("Begin authorizing an account with `auth` first.".to_owned());
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
                    // TODO: profile names?
                    /*
                     * Option 1:
                     *  ask user.
                     *  yes, but I want this to be optional though (auth, pin 1234, profile now
                     *  named main or after you or something)
                     * Option 2:
                     *  make a request for profile settings when auth succeeds
                     *  this becomes the fallback when nothing is provided in option 1
                     *  what happens when you successfully auth, internet drops, and you fail to
                     *  request settings?
                     *
                     *  fallback to asking user to name the profile, i guess?
                     */
                    if tweeter.curr_profile.is_none() {
                        tweeter.curr_profile = Some("default".to_owned());
                    }
                    let user_credential = tw::Credential {
                        key: as_map["oauth_token"].to_owned(),
                        secret: as_map["oauth_token_secret"].to_owned()
                    };

                    match queryer.do_api_get(::ACCOUNT_SETTINGS_URL, &tweeter.app_key, &user_credential) {
                        Ok(settings) => {
                            let user_handle = settings["screen_name"].as_str().unwrap().to_owned();
                            tweeter.add_profile(tw::TwitterProfile::new(user_credential, tw::user::User::default()), Some(user_handle.clone()), display_info);
                            tweeter.WIP_auth = None;
                            tweeter.state = tw::AppState::Reconnect(user_handle);
                            display_info.status("Looks like you authed! Connecting...".to_owned());
                        },
                        Err(_) => {
                            display_info.status("Auth failed - couldn't find your handle.".to_owned());
                        }
                    };
                },
                Err(_) =>
                    display_info.status("couldn't rebuild url".to_owned())
            },
        Err(e) =>
            display_info.status(format!("request token url error: {}", e))
    };
}
