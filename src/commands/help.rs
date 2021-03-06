use display::DisplayInfo;
use tw;
use ::Queryer;

use commands::Command;

pub static HELP: Command = Command {
    keyword: "help",
    params: 0,
    exec: help,
    param_str: "",
    help_str: "This help prompt."
};

fn help(line: String, tweeter: &mut tw::TwitterCache, queryer: &mut Queryer, display_info: &mut DisplayInfo) {
    tweeter.state = tw::AppState::ShowHelp;
}
