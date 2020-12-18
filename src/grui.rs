use std::collections::HashMap;

use iced::{
    executor, Application, Button, Column, Command, Container, Element, Length, Row, Settings,
    Subscription, Text,
};

use crate::command_actions::{GrunnerConfig, GrunnerOption};
use crate::{command_actions::GrunnerAction, task_subscription};

pub fn run_grui(config: GrunnerConfig) {
    Grui::run(Settings::with_flags(config)).expect("Error running Grunner UI");
}

#[derive(Debug)]
pub struct Grui {
    config: GrunnerConfig,
    state: GState,
    button_states: HashMap<String, iced::button::State>,
}

#[derive(Debug)]
enum GState {
    Idle,
    Working(GrunnerAction),
}

#[derive(Debug, Clone)]
pub enum GruiMessage {
    _Start,
    StartAction(GrunnerAction),
    ActionUpdate(task_subscription::ActionProgress),
    OptionChanged(String, GrunnerOptionMessage),
}

#[derive(Debug, Clone)]
pub enum GrunnerOptionMessage {
    ChoiceChanged(usize),
    FlagChanged(bool),
}

//------------------------------------------------------------------------------
// impl GrunnerOption
//------------------------------------------------------------------------------
impl GrunnerOption {
    fn update(&mut self, message: GrunnerOptionMessage) {
        match message {
            GrunnerOptionMessage::ChoiceChanged(id) => {
                if let GrunnerOption::Choices {
                    choices: _,
                    selected,
                } = self
                {
                    *selected = Some(id);
                }
            }
            GrunnerOptionMessage::FlagChanged(val) => {
                if let GrunnerOption::Flag {
                    name: _,
                    value,
                    arg: _,
                } = self
                {
                    *value = val;
                }
            }
        }
    }

    fn view(&mut self) -> Element<GrunnerOptionMessage> {
        // TODO: Flesh out (move/add as needed)
        match self {
            GrunnerOption::Choices { choices, selected } => {
                let mut content = Row::new().spacing(8);
                for choice in choices.iter() {
                    // Check for unset option and then just select the first one
                    // TODO: Include an optional default option and set that (somewhere) if it exists
                    if let None = selected {
                        *selected = Some(choice.id);
                    }
                    content = content.push(iced::radio::Radio::new(
                        choice.id,
                        &choice.label,
                        selected.to_owned(),
                        GrunnerOptionMessage::ChoiceChanged,
                    ));
                }
                content.into()
            }
            GrunnerOption::Flag {
                name,
                value,
                arg: _,
            } => {
                let content = iced::checkbox::Checkbox::new(
                    *value,
                    name.to_owned(),
                    GrunnerOptionMessage::FlagChanged,
                );
                content.into()
            }
        }
    }
}

//------------------------------------------------------------------------------
// impl Grui
//------------------------------------------------------------------------------
impl Grui {
    fn new(config: GrunnerConfig) -> Self {
        Grui {
            config,
            state: GState::Idle,
            button_states: HashMap::new(),
        }
    }
}

//------------------------------------------------------------------------------
// impl Application for Grui
//------------------------------------------------------------------------------
impl Application for Grui {
    type Executor = executor::Default;
    type Message = GruiMessage;
    type Flags = GrunnerConfig;

    fn new(flags: GrunnerConfig) -> (Grui, Command<GruiMessage>) {
        (Grui::new(flags), Command::none())
    }

    fn title(&self) -> String {
        String::from("Grunner UI")
    }

    fn update(&mut self, message: GruiMessage) -> Command<GruiMessage> {
        // TODO: Change messages to refer to an index in the list of options and then forward
        match message {
            GruiMessage::_Start => {}
            GruiMessage::StartAction(act) => self.state = GState::Working(act),
            GruiMessage::ActionUpdate(update) => match update {
                task_subscription::ActionProgress::Starting => {}
                task_subscription::ActionProgress::Continuing => {}
                task_subscription::ActionProgress::Completed => self.state = GState::Idle,
                task_subscription::ActionProgress::Error => self.state = GState::Idle,
            },
            GruiMessage::OptionChanged(name, opt_message) => {
                'top: for sect in &mut self.config.sections {
                    for (opt_name, opt) in &mut sect.options {
                        if *opt_name != name {
                            continue;
                        }

                        opt.update(opt_message);
                        break 'top;
                    }
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<GruiMessage> {
        match &self.state {
            GState::Working(act) => {
                task_subscription::build_subscription(&act).map(GruiMessage::ActionUpdate)
            }
            _ => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<GruiMessage> {
        let content = match self.state {
            GState::Idle => {
                let sections: Element<_> = self
                    .config
                    .sections
                    .iter_mut()
                    .fold(Column::new().spacing(20), |content, section| {
                        // TODO: Draw a label and separator for this section

                        let options_gui: Element<_> = section
                            .options
                            .iter_mut()
                            .fold(Column::new().spacing(8), |column, (opt_name, opt)| {
                                let owned_name = opt_name.clone();
                                column.push(opt.view().map(move |msg| {
                                    GruiMessage::OptionChanged(owned_name.clone(), msg)
                                }))
                            })
                            .into();

                        let actions_gui: Element<_> = section
                            .actions
                            .iter_mut()
                            .fold(Row::new().spacing(8), |content, (text, act)| {
                                let act_clone = act.clone();
                                content.push(
                                    Button::new(&mut act.gui_state, Text::new(text))
                                        .on_press(GruiMessage::StartAction(act_clone)),
                                )
                            })
                            .into();

                        content.push(options_gui).push(actions_gui)
                    })
                    .into();

                sections
            }
            GState::Working(_) => Column::new().push(Text::new("Working...")).into(),
        };
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
