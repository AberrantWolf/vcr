use std::process::Stdio;

use iced_futures::futures;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStderr, ChildStdout, Command},
};

use crate::command_actions::GrunnerAction;

pub fn build_subscription(action: &GrunnerAction) -> iced::Subscription<ActionProgress> {
    iced::Subscription::from_recipe(DoAction {
        action: action.clone(),
    })
}

pub struct DoAction {
    action: GrunnerAction,
}

pub enum ActionState {
    Ready(GrunnerAction),
    InProgress(
        BufReader<ChildStdout>,
        BufReader<ChildStderr>,
        Child,
        GrunnerAction,
    ),
    Done,
}

#[derive(Debug, Clone, Copy)]
pub enum ActionResult {
    Success,
    Fail,
}

#[derive(Debug, Clone)]
pub enum ActionProgress {
    Starting,
    Continuing,
    Completed(ActionResult),
    Error,
}

fn run_async_process(action: &GrunnerAction) -> (ChildStdout, ChildStderr, Child) {
    println!("Running action...");
    println!("-----------------");

    let mut proc = Command::new(action.execute.clone())
        .args(action.args.clone())
        .args(action.options.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("Unable to spawn async process. :(");
    let out = proc
        .stdout
        .take()
        .expect("Couldn't take the process stdout?!");
    let err = proc
        .stderr
        .take()
        .expect("Couldn't take the process stderr?!");
    (out, err, proc)
}

impl<H, I> iced_native::subscription::Recipe<H, I> for DoAction
where
    H: std::hash::Hasher,
{
    type Output = ActionProgress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state)
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            ActionState::Ready(self.action.clone()),
            |state| async move {
                match state {
                    ActionState::Ready(action) => {
                        let (stdout, stderr, async_child) = run_async_process(&action);
                        let out_reader = BufReader::new(stdout);
                        let err_reader = BufReader::new(stderr);

                        Some((
                            ActionProgress::Starting,
                            ActionState::InProgress(out_reader, err_reader, async_child, action),
                        ))
                    }
                    ActionState::InProgress(mut out_reader, mut err_reader, mut child, action) => {
                        let mut out_line = String::new();
                        let mut err_line = String::new();

                        let (readline, line) = tokio::select! {
                            read = out_reader.read_line(&mut out_line) => (read, out_line),
                            read = err_reader.read_line(&mut err_line) => (read, err_line),
                        };

                        match readline {
                            Ok(count_bytes) => {
                                if count_bytes == 0 {
                                    let child_result = child
                                        .wait()
                                        .await
                                        .expect("Error waiting for child process to complete");

                                    let result = if child_result.success() {
                                        // TODO: Play succeed sound if one is set
                                        println!("Action succeeded!");
                                        ActionResult::Success
                                    } else {
                                        // TODO: Play fail sound if one is set
                                        println!("Action failed!");
                                        ActionResult::Fail
                                    };

                                    println!("-----------------");
                                    return Some((
                                        ActionProgress::Completed(result),
                                        ActionState::Done,
                                    ));
                                }

                                print!("{}", &line);
                                // TODO: Make a log view to display this info in the app window

                                Some((
                                    ActionProgress::Continuing,
                                    ActionState::InProgress(out_reader, err_reader, child, action),
                                ))
                            }
                            Err(err) => {
                                println!("ERROR: {}", err);
                                Some((ActionProgress::Error, ActionState::Done))
                            }
                        }
                    }
                    ActionState::Done => None,
                }
            },
        ))
    }
}
