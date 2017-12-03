use display::DisplayInfo;
use tw;
use ::Queryer;

use commands::Command;

pub static PROFILE: Command = Command {
    keyword: "profile",
    params: 1,
    exec: switch_profile,
    param_str: " <profile_name>",
    help_str: "Switch to profile <profile_name>"
};

fn switch_profile(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    let profile_name = line.trim();
    if tweeter.profiles.contains_key(profile_name) {
        tweeter.curr_profile = Some(profile_name.to_owned());
    } else {
        display_info.status(format!("No profile named {}", profile_name))
    };
}

pub static PROFILES: Command = Command {
    keyword: "profiles",
    params: 0,
    exec: list_profiles,
    param_str: "",
    help_str: "List all profiles"
};

fn list_profiles(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    display_info.recv(::display::Infos::Text(
        tweeter.profiles.keys().map(|key| key.to_owned()).collect()
    ));
}
