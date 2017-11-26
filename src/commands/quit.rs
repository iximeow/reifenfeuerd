use display::DisplayInfo;
use tw;
use ::Queryer;

use commands::Command;

pub static QUIT: Command = Command {
    keyword: "q",
    params: 0,
    exec: quit,
    param_str: "",
    // TODO: app name
    help_str: "Gracefully exit this thing"
};

fn quit(_line: String, tweeter: &mut tw::TwitterCache, _queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    tweeter.state = tw::AppState::Shutdown;
}
