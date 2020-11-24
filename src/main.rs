mod command_actions;
mod grui;

use command_actions::{load_grunner_config, CommandAction};

use grui::run_grui;

use std::io::{self, Write};
use std::process::Command;

fn run_action_sync(action: &CommandAction) {
    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
    } else {
        Command::new("sh")
    }
    .arg(&action.execute)
    .output()
    .expect("Failed to execute action");

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    // let result = String::from_utf8(output.stdout).expect("Failed to create string from UTF8");
    // println!("Output {}", result);
}

fn main() {
    let config = load_grunner_config("grunner.json").unwrap();

    run_grui(config);
}
