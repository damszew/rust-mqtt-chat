use anyhow::Result;
use tokio::sync::mpsc;

use crate::{
    renderer::{Renderer, State},
    AppEvent,
};

pub struct App<R> {
    receiver: mpsc::Receiver<AppEvent>,
    renderer: R,
    state: State,
}

impl<R> App<R>
where
    R: Renderer,
{
    pub fn new(receiver: mpsc::Receiver<AppEvent>, mut renderer: R) -> Self {
        let state = State::default();
        renderer.render(&state).unwrap();

        Self {
            receiver,
            renderer,
            state,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(event) = self.receiver.recv().await {
            match event {
                AppEvent::Quit => {
                    break;
                }
                AppEvent::Character(ch) => {
                    self.state.input_message.push(ch);
                    self.renderer.render(&self.state)?;
                }
                _ => todo!(),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod should {
    use std::time::Duration;

    use mockall::predicate::*;
    use tokio::{sync::mpsc, task};

    use super::*;

    use crate::renderer::{MockRenderer, State};

    #[test]
    fn render_frame_on_startup() {
        let renderer_mock = setup_rendered_mock();

        let (_, receiver) = mpsc::channel(1);
        let _ = App::new(receiver, renderer_mock);
    }

    #[tokio::test]
    async fn render_frame_on_update() {
        let event = AppEvent::Character('c');
        let expected_state = State {
            input_message: "c".into(),
            ..Default::default()
        };

        let mut renderer_mock = setup_rendered_mock();
        renderer_mock
            .expect_render()
            .times(1)
            .with(eq(expected_state))
            .returning(|_| Ok(()));

        let (sender, receiver) = mpsc::channel(1);
        let mut tested_app = App::new(receiver, renderer_mock);

        task::spawn(async move {
            sender.send(event).await.unwrap();
        });

        tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await.unwrap();
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn exit_loop_on_quit() {
        let event = AppEvent::Quit;

        let renderer_mock = setup_rendered_mock();

        let (sender, receiver) = mpsc::channel(1);
        let mut tested_app = App::new(receiver, renderer_mock);

        let test_sender = sender.clone();
        task::spawn(async move {
            test_sender.send(event).await.unwrap();
        });

        tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await.unwrap();
        })
        .await
        .unwrap();
    }

    fn setup_rendered_mock() -> MockRenderer {
        let mut renderer_mock = MockRenderer::new();

        // start-up call
        renderer_mock
            .expect_render()
            .times(1)
            .with(eq(State::default()))
            .returning(|_| Ok(()));

        renderer_mock
    }
}
