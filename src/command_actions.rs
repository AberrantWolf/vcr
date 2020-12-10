use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandAction {
    pub execute: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(skip)]
    pub gui_state: iced::button::State,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrunnerConfig {
    pub actions: HashMap<String, CommandAction>,
}

pub fn load_grunner_config<S>(path: S) -> Result<GrunnerConfig>
where
    S: Into<String>,
{
    let mut file = File::open(path.into()).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let config: GrunnerConfig = serde_json::from_str(&data).unwrap();

    Ok(config)
}
