mod command_actions;

use command_actions::load_grunner_config;

fn main() {
    let config = load_grunner_config("grunner.json").unwrap();

    for (name, act) in config.actions {
        println!("ACTION: {:?}: {:?}", name, act.execute);
    }
}
