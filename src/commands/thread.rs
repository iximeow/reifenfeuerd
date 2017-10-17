use tw;
use ::Queryer;
use ::display;

use commands::Command;

use std::str::FromStr;

pub static FORGET_THREAD: Command = Command {
    keyword: "forget",
    params: 1,
    exec: forget
};

fn forget(line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    tweeter.forget_thread(line.trim().to_string());
    println!("Ok! Forgot thread {}", line.trim().to_string());
}

pub static REMEMBER_THREAD: Command = Command {
    keyword: "remember",
    params: 2,
    exec: remember
};

fn remember(line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    let mut text: String = line.trim().to_string();
    if let Some(id_end_idx) = text.find(" ") {
        let name_bare = text.split_off(id_end_idx + 1);
        let name = name_bare.trim();
        let id_str = text.trim();
        if name.len() > 0 {
            if let Some(inner_twid) = u64::from_str(&id_str).ok() {
                if tweeter.tweet_by_innerid(inner_twid).is_some() {
                    tweeter.set_thread(name.to_string(), inner_twid);
                    println!("Ok! Recorded {} as thread {}", inner_twid, name);
                }
            }
        }
    }
}

pub static LIST_THREADS: Command = Command {
    keyword: "ls_threads",
    params: 0,
    exec: ls_threads
};

fn ls_threads(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    println!("Threads: ");
    for k in tweeter.threads() {
        println!("Thread: {}", k);
        let latest_inner_id = tweeter.latest_in_thread(k.to_owned()).unwrap();
        let twete = tweeter.tweet_by_innerid(*latest_inner_id).unwrap();
                                // gross..
        display::render_twete(&twete.id, tweeter);
        println!("");
    }
}
