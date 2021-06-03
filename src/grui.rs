use std::collections::HashMap;
use std::sync::mpsc::Sender;

use iced::{
    executor, Align, Application, Button, Clipboard, Column, Command, Container, Element, Font,
    Length, Row, Rule, Settings, Subscription, Text,
};

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

use crate::command_actions::{GrunnerConfig, GrunnerOption};
use crate::{command_actions::GrunnerAction, task_subscription};

pub fn run_grui(config: GrunnerConfig) {
    Grui::run(Settings::with_flags(config)).expect("Error running VCR UI");
}

#[derive(Debug)]
pub struct Grui {
    config: GrunnerConfig,
    state: GState,
    sender: Sender<String>,
}

#[derive(Debug)]
enum GState {
    Idle,
    Working(GrunnerAction),
}

#[derive(Debug, Clone)]
pub enum GruiMessage {
    _Start,
    StartAction(usize, GrunnerAction),
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
                    name: _,
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
                    label: _,
                    value,
                    args: _,
                } = self
                {
                    *value = val;
                }
            }
        }
    }

    fn view(&mut self) -> Element<GrunnerOptionMessage> {
        match self {
            GrunnerOption::Choices {
                name: _,
                choices,
                selected,
            } => {
                let mut content = Row::new().spacing(8);
                for (idx, choice) in choices.iter().enumerate() {
                    // Check for unset option and then just select the first one
                    // TODO: Include an optional default option and set that (somewhere) if exists
                    if let None = selected {
                        *selected = Some(idx);
                    }
                    content = content.push(iced::radio::Radio::new(
                        idx,
                        &choice.label,
                        selected.to_owned(),
                        GrunnerOptionMessage::ChoiceChanged,
                    ));
                }
                content.into()
            }
            GrunnerOption::Flag {
                name: _,
                label,
                value,
                args: _,
            } => {
                let content = iced::checkbox::Checkbox::new(
                    *value,
                    label.to_owned(),
                    GrunnerOptionMessage::FlagChanged,
                );
                content.into()
            } // TODO: Input for a user to just type whatever?
        }
    }
}

//------------------------------------------------------------------------------
// impl Grui
//------------------------------------------------------------------------------
impl Grui {
    fn new(config: GrunnerConfig) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<String>();

        std::thread::spawn(move || {
            let (_audio_stream, audio_stream_handle) = OutputStream::try_default().unwrap();

            while let Ok(path) = receiver.recv() {
                if let Ok(file) = File::open(&path) {
                    let buf_reader = BufReader::new(file);
                    if let Ok(source) = Decoder::new(buf_reader) {
                        audio_stream_handle
                            .play_raw(source.convert_samples())
                            .expect("unable to play decoded audio");
                    } else {
                        println!("Error decoding audio file: {}", path);
                    }
                } else {
                    println!("Error opening audio file: {}", path);
                }
            }
        });

        Grui {
            config,
            state: GState::Idle,
            sender,
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
        String::from("VCR -- The Visual Command Runner")
    }

    fn update(&mut self, message: GruiMessage, _clipboard: &mut Clipboard) -> Command<GruiMessage> {
        match message {
            GruiMessage::_Start => {}
            GruiMessage::StartAction(sect_idx, mut act) => {
                // Collect enabled options' args and set the options list from them
                let mut opts: Vec<String> = vec![];
                let mut reps = HashMap::<String, String>::new();
                for opt in act.use_options.iter() {
                    // Look for the option name in the list of options
                    if let Some(&gopt) = self.config.sections[sect_idx]
                        .options
                        .iter()
                        .filter(|opti| match opti {
                            GrunnerOption::Choices {
                                name,
                                choices: _,
                                selected: _,
                            } => name == opt,
                            GrunnerOption::Flag {
                                name,
                                label: _,
                                value: _,
                                args: _,
                            } => name == opt,
                        })
                        .collect::<Vec<&GrunnerOption>>()
                        .first()
                    {
                        opts.extend(gopt.get_arg().iter().map(|o| o.clone()));

                        for (k, v) in gopt.get_replacements().iter() {
                            reps.insert(k.clone(), v.clone());
                        }
                    }
                }

                // println!("OPTIONS: {:?}", opts);
                act.set_selected_options(opts);
                act.apply_replacement_map(&reps);
                self.state = GState::Working(act);
            }
            GruiMessage::ActionUpdate(update) => match update {
                task_subscription::ActionProgress::Starting => {}
                task_subscription::ActionProgress::Continuing => {}
                task_subscription::ActionProgress::Completed(result) => {
                    // Play sound here
                    if let GState::Working(state) = &self.state {
                        match result {
                            task_subscription::ActionResult::Success => {
                                // Play success sound
                                if let Some(fname) = &state.success_sound {
                                    self.sender
                                        .send(fname.clone())
                                        .expect("Unable to send filename to audio thread");
                                }
                            }
                            task_subscription::ActionResult::Fail => {
                                // Play fail sound
                                if let Some(fname) = &state.fail_sound {
                                    self.sender
                                        .send(fname.clone())
                                        .expect("Unable to send filename to audio thread");
                                }
                            }
                        }
                    }

                    self.state = GState::Idle
                }
                task_subscription::ActionProgress::Error => self.state = GState::Idle,
            },
            GruiMessage::OptionChanged(name, opt_message) => {
                'top: for sect in &mut self.config.sections {
                    for opt in &mut sect.options {
                        if opt.get_name() != name {
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
                    .enumerate()
                    .fold(Column::new().spacing(20), |content, (sect_idx, section)| {
                        let header = Row::new()
                            .push(Rule::horizontal(0))
                            .push(Text::new(&section.label).font(Font::Default))
                            .push(Rule::horizontal(0))
                            .align_items(Align::Center)
                            .spacing(0);

                        let options_gui: Element<_> = section
                            .options
                            .iter_mut()
                            .fold(Column::new().spacing(8), |column, opt| {
                                // NOTE: This clone and the one below seem to be needed in order
                                // to know the lifetime. I'm not sure why rustc can't figure it
                                // out, but it definitely doesn't like just cloning `opt_name`
                                // into the enum below.
                                let owned_name = opt.get_name().to_owned();
                                column.push(opt.view().map(move |msg| {
                                    GruiMessage::OptionChanged(owned_name.clone(), msg)
                                }))
                            })
                            .into();

                        let actions_gui: Element<_> = section
                            .actions
                            .iter_mut()
                            .fold(Row::new().spacing(8), |content, act| {
                                let act_clone = act.clone();
                                // println!("SECT: {}, {}", sect_idx, text);
                                content.push(
                                    Button::new(&mut act.gui_state, Text::new(act.name.to_owned()))
                                        .on_press(GruiMessage::StartAction(sect_idx, act_clone)),
                                )
                            })
                            .into();

                        content.push(header).push(options_gui).push(actions_gui)
                    })
                    .align_items(Align::Center)
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
