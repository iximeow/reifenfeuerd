use tw;
use ::Queryer;
use ::display;

use tw::TweetId;

use commands::Command;

pub static FORGET_THREAD: Command = Command {
    keyword: "forget",
    params: 1,
    exec: forget,
    help_str: "<name>: Discard thread known by <name>. Entirely local to the client."
};

fn forget(line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    tweeter.forget_thread(line.trim().to_string());
    tweeter.display_info.status(format!("Ok! Forgot thread {}", line.trim().to_string()));
}

pub static REMEMBER_THREAD: Command = Command {
    keyword: "remember",
    params: 2,
    exec: remember,
    help_str: "<tweet_id> <name>: Remember the thread tipped by <tweet_id> as  <name>. Entirely local to the client."
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
                Ok(twid) => {
                    if let Some(twete) = tweeter.retrieve_tweet(&twid).map(|x| x.clone()) {
                        tweeter.set_thread(name.to_string(), twete.internal_id);
                        tweeter.display_info.status(format!("Ok! Recorded {:?} as thread {}", twid, name));
                    } else {
                        tweeter.display_info.status(format!("No tweet for id: {:?}", twid));
                    }
                }
                Err(e) => {
                    tweeter.display_info.status(format!("Invalid id: {}", e));
                }
            }
        }
    }
}

pub static LIST_THREADS: Command = Command {
    keyword: "ls_threads",
    params: 0,
    exec: ls_threads,
    help_str: "Show all known (local) threads"
};

fn ls_threads(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer) {
    let threads: Vec<String> = tweeter.threads().collect::<Vec<&String>>().into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
    for k in threads {
        let latest_inner_id = tweeter.latest_in_thread(k.to_owned()).unwrap().to_owned();
        tweeter.display_info.recv(display::Infos::TweetWithContext(TweetId::Bare(latest_inner_id), format!("Thread: {}", k)))
    }
}
