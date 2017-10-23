use tw;
use ::Queryer;
use ::display;

use tw::TweetId;

use commands::Command;

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
                Ok(twid) => {
                    let twete = tweeter.retrieve_tweet(&twid).unwrap().clone();
                    tweeter.set_thread(name.to_string(), twete.internal_id);
                    println!("Ok! Recorded {:?} as thread {}", twid, name);
                }
                Err(e) => {
                    println!("Invalid id: {}", e);
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
    let threads: Vec<String> = tweeter.threads().collect::<Vec<&String>>().into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
    for k in threads {
        println!("Thread: {}", k);
        let latest_inner_id = tweeter.latest_in_thread(k.to_owned()).unwrap().to_owned();
        // should be able to just directly render TweetId.. and threads should be Vec<TweetId>...
        let twete_id_TEMP = tweeter.retrieve_tweet(&TweetId::Bare(latest_inner_id)).map(|x| x.id.to_owned());
        if let Some(twete) = twete_id_TEMP {
                                // gross..
            // and this ought to be a command to tweeter.display_info anyway...
            display::render_twete(&TweetId::Twitter(twete), tweeter);
            println!("");
        } else {
            println!("ERROR no tweet for remembered thread.");
        }
    }
}
