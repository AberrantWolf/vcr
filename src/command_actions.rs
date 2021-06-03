use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use string_template::Template;

thread_local!(static OPTION_ID: Cell<usize> = Cell::new(0));
fn next_option_id() -> usize {
    OPTION_ID.with(|id| {
        let result = id.get();
        id.set(result + 1);
        result
    })
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GrunnerChoiceType {
    #[serde(skip, default = "next_option_id")]
    pub id: usize,
    pub label: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub replacements: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub struct GrunnerOption {
    pub(crate) name: String,
    pub(crate) choices: Vec<GrunnerChoiceType>,

    #[serde(skip)]
    pub(crate) selected: Option<usize>,
}

impl GrunnerOption {
    pub fn get_choice(&self) -> GrunnerChoiceType {
        if let Some(idx) = self.selected {
            self.choices[idx].clone()
        } else {
            GrunnerChoiceType::default()
        }
    }

    pub fn get_choices(&self) -> &Vec<GrunnerChoiceType> {
        &self.choices
    }

    pub fn get_arg(&self) -> Vec<String> {
        self.get_choice().args
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_replacements(&self) -> HashMap<String, String> {
        self.get_choice().replacements
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrunnerAction {
    pub name: String,
    pub execute: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub use_options: Vec<String>,

    #[serde(default)]
    pub success_sound: Option<String>,

    #[serde(default)]
    pub fail_sound: Option<String>,

    #[serde(skip)]
    pub options: Vec<String>,

    #[serde(skip)]
    pub gui_state: iced::button::State,
}

impl GrunnerAction {
    pub fn set_selected_options(&mut self, opts: Vec<String>) {
        self.options = opts;
    }

    pub fn apply_replacement_map(&mut self, reps: &HashMap<String, String>) {
        // Format necessary strings based on selected options
        let fixed_map: HashMap<&str, &str> = reps
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect::<HashMap<&str, &str>>();

        let template = Template::new(&self.execute);
        let new_execute = template.render(&fixed_map);

        self.execute = new_execute;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrunnerSection {
    pub label: String,

    #[serde(default)]
    pub options: Vec<GrunnerOption>,
    pub actions: Vec<GrunnerAction>,
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
