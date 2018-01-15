use display::DisplayInfo;
use tw;
use ::Queryer;

use tw::TweetId;

use commands::Command;

static DM_CREATE_URL: &str = "https://api.twitter.com/1.1/direct_messages/new.json";

pub static DM: Command = Command {
    keyword: "dm",
    params: 1,
    exec: dm,
    param_str: " <user_handle>",
    help_str: "Send DM to <user_handle>"
};

fn dm(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let user_profile = match tweeter.current_profile().map(|profile| profile.to_owned()) {
        Some(profile) => profile,
        None => {
            display_info.status("To send a DM you must be an authenticated user.".to_owned());
            return;
        }
    };
    let mut text: String = line.trim().to_string();
    let text_bare = match text.find(" ") {
        None => "".to_owned(),
        Some(id_end_idx) => {
            text.split_off(id_end_idx + 1)
        }
    };
    let dm_text = text_bare.trim();
    let handle_chars = text.trim().chars().collect::<Vec<char>>();
    let normalized_handle = if handle_chars[0] == '@' {
        handle_chars[1..].to_vec()
    } else {
        handle_chars
    }.into_iter().collect::<String>();

    let encoded = ::url_encode(dm_text);
    let result = match tweeter.current_profile() {
        Some(user_profile) => {
            queryer.do_api_post(
                DM_CREATE_URL,
                &vec![("text", &encoded), ("screen_name", &normalized_handle)],
                &tweeter.app_key,
                &user_profile.creds
            )
        },
        None => Err("No logged in user to DM as".to_owned())
    };
    match result {
        Ok(_) => (),
        Err(e) => display_info.status(e)
    }
}
