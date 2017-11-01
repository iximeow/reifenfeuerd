use tw;
use ::Queryer;

use commands::Command;

use std::process::exit;

pub static QUIT: Command = Command {
    keyword: "q",
    params: 0,
    exec: quit
};

fn quit(_line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    tweeter.display_info.status("Bye bye!".to_owned());
    tweeter.store_cache();
    exit(0);
}
