use tw;
use ::Queryer;

pub struct Command {
    pub keyword: &'static str,
    pub params: u8,
    pub exec: fn(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer),
    pub param_str: &'static str,
    pub help_str: &'static str
}

pub mod help;
pub mod auth;
pub mod show_cache;
pub mod twete;
pub mod look_up;
pub mod view;
pub mod quit;
pub mod fav;
pub mod follow;
pub mod thread;

pub static COMMANDS: &[&Command] = &[
    &help::HELP,
    &auth::AUTH,
    &auth::PIN,
    &show_cache::SHOW_CACHE,
    &quit::QUIT,
    &look_up::LOOK_UP_USER,
    &look_up::LOOK_UP_TWEET,
    &view::VIEW,
    &view::VIEW_THREAD,
    &follow::FOLLOW,
    &follow::UNFOLLOW,
    &fav::UNFAV,
    &fav::FAV,
    &twete::DEL,
    &twete::TWETE,
    &twete::QUOTE,
    &twete::RETWETE,
    &twete::REP,
    &twete::THREAD,
    &thread::FORGET_THREAD,
    &thread::REMEMBER_THREAD,
    &thread::LIST_THREADS
];
