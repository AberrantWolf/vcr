use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

thread_local!(static OPTION_ID: Cell<usize> = Cell::new(0));
fn next_option_id() -> usize {
    OPTION_ID.with(|id| {
        let result = id.get();
        id.set(result + 1);
        result
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrunnerChoiceType {
    #[serde(skip, default = "next_option_id")]
    pub id: usize,
    pub label: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GrunnerOption {
    Choices {
        choices: Vec<GrunnerChoiceType>,

        #[serde(skip)]
        selected: Option<usize>,
    },
    Flag {
        name: String,
        value: bool,
        args: Vec<String>,
    },
}

impl GrunnerOption {
    pub fn get_arg(&self) -> Vec<String> {
        // println!("OPTION: {:?}", self);
        match self {
            GrunnerOption::Choices { choices, selected } => {
                if let Some(idx) = selected {
                    let args = &choices[*idx].args;
                    if args.is_empty() {
                        vec![]
                    } else {
                        args.clone().into()
                    }
                } else {
                    vec![]
                }
            }
            GrunnerOption::Flag {
                name: _,
                value,
                args,
            } if *value => args.clone().into(),
            _ => vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrunnerSection {
    pub label: String,

    #[serde(default)]
    pub options: HashMap<String, GrunnerOption>,
    pub actions: HashMap<String, GrunnerAction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrunnerAction {
    pub execute: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub use_options: Vec<String>,

    #[serde(skip)]
    pub options: Vec<String>,

    #[serde(skip)]
    pub gui_state: iced::button::State,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrunnerConfig {
    pub sections: Vec<GrunnerSection>,
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
