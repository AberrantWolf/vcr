use std::collections::HashMap;

use iced::{
    executor, Application, Button, Column, Command, Container, Element, Length, Settings,
    Subscription, Text,
};

use crate::command_actions::GrunnerConfig;
use crate::{command_actions::CommandAction, task_subscription};

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
    Working(CommandAction),
}

#[derive(Debug, Clone)]
pub enum GruiMessage {
    Start,
    StartAction(CommandAction),
    ActionUpdate(task_subscription::ActionProgress),
}

impl Grui {
    fn new(config: GrunnerConfig) -> Self {
        Grui {
            config,
            state: GState::Idle,
            button_states: HashMap::new(),
        }
    }
}

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
        match message {
            GruiMessage::Start => {}
            GruiMessage::StartAction(act) => self.state = GState::Working(act),
            GruiMessage::ActionUpdate(update) => match update {
                task_subscription::ActionProgress::Starting => {}
                task_subscription::ActionProgress::Continuing => {}
                task_subscription::ActionProgress::Completed => self.state = GState::Idle,
                task_subscription::ActionProgress::Error => self.state = GState::Idle,
            },
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
        let mut content = Column::new();

        match self.state {
            GState::Idle => {
                for (text, act) in self.config.actions.iter_mut() {
                    let act_clone = act.clone();
                    content = content.push(
                        Button::new(&mut act.gui_state, Text::new(text))
                            .on_press(GruiMessage::StartAction(act_clone)),
                    );
                }
                // let column: Column<_> = self
                //     .config
                //     .actions
                //     .iter()
                //     .enumerate()
                //     .fold(Column::new(), |column, (i, (text, act))| {
                //         column.push(
                //             Button::new(btn_state, Text::new(text))
                //                 .on_press(GruiMessage::StartAction(act.clone())),
                //         )
                //     })
                //     .into();
            }
            GState::Working(_) => {
                content = content.push(Text::new("Working..."));
            }
        }
        // let column: Column<_> = self
        //     .config
        //     .actions
        //     .iter()
        //     .enumerate()
        //     .fold(Column::new(), |column, (i, (text, _act))| {
        //         column.push(Text::new(text))
        //     })
        //     .into();

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
