use tw;
use ::Queryer;

use commands::Command;

use std::process::exit;

pub static QUIT: Command = Command {
    keyword: "q",
    params: 0,
    exec: quit,
    // TODO: app name
    help_str: "Gracefully exit this thing"
};

fn quit(_line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer) {
    tweeter.state = tw::AppState::Shutdown;
}
