use tw;
use ::Queryer;

pub struct Command {
    pub keyword: &'static str,
    pub params: u8,
    pub exec: fn(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer)
}

mod show_cache;
mod twete;
mod look_up;
mod view;
mod quit;
mod fav;
mod follow;
mod thread;

pub static COMMANDS: &[&Command] = &[
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
    /*
        &QUIT,
        &LOOK_UP_USER,
        &LOOK_UP_TWEET,
        &VIEW,
        &UNFAV,
        &FAV,
        &DEL,
        &TWETE,
        &QUOTE,
        &RETWETE,
        &REP,
        &THREAD
    ];
    */
];
