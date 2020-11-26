use iced_futures::futures;

use crate::command_actions::CommandAction;

pub struct DoAction {
    action: CommandAction,
}

pub enum ActionState {
    Ready,
}

#[derive(Debug, Clone)]
pub enum ActionProgress {}

impl<H, I> iced_native::subscription::Recipe<H, I> for DoAction
where
    H: std::hash::Hasher,
{
    type Output = ActionProgress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state)
    }

    fn stream(self: Box<Self>, _input: futures::stream::BoxStream<'static, I>,) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            ActionState::Ready,
            |state| async move {
                match state {
                    ActionState::Ready => {
                        None
                    }
                }
            },
        ))
    }
}