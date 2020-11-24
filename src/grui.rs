use iced::{
    executor, Application, Column, Command, Container, Element, Length, Settings, Subscription,
    Text,
};

use super::command_actions::GrunnerConfig;

pub fn run_grui(config: GrunnerConfig) {
    Grui::run(Settings::with_flags(config)).expect("Error running Grunner UI");
}

#[derive(Debug)]
pub struct Grui {
    config: GrunnerConfig,
}

#[derive(Debug, Clone)]
pub enum GruiMessage {
    Start,
}

impl Grui {
    fn new(config: GrunnerConfig) -> Self {
        Grui { config }
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
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<GruiMessage> {
        Subscription::none()
    }

    fn view(&mut self) -> Element<GruiMessage> {
        let mut content = Column::new();

        // let column: Column<_> = self
        //     .config
        //     .actions
        //     .iter()
        //     .enumerate()
        //     .fold(Column::new(), |column, (i, (text, _act))| {
        //         column.push(Text::new(text))
        //     })
        //     .into();
        for (text, _act) in &self.config.actions {
            content = content.push(Text::new(text));
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
