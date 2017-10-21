use tw;
use ::Queryer;
use ::display;

use tw::TweetId;

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
            let maybe_id = TweetId::parse(line.to_owned());
            match maybe_id {
                Some(twid) => {
                    let twete = tweeter.retrieve_tweet(&twid).unwrap().clone();
                    tweeter.set_thread(name.to_string(), twete.internal_id);
                    println!("Ok! Recorded {:?} as thread {}", twid, name);
                }
                None => {
                    println!("Invalid id: {}", line);
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
        if let Some(twete) = tweeter.retrieve_tweet(&TweetId::Bare(*latest_inner_id)) {
                                // gross..
            display::render_twete(&twete.id, tweeter);
            println!("");
        } else {
            println!("ERROR no tweet for remembered thread.");
        }
    }
}
