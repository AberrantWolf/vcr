mod command_actions;
mod grui;
mod task_subscription;

use command_actions::load_grunner_config;

use grui::run_grui;

fn main() {
    let config = load_grunner_config("grunner.json").unwrap();

    run_grui(config);
}
