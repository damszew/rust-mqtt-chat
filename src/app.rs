use tokio::sync::mpsc;

use crate::renderer::{Renderer, State};

pub struct App<R> {
    receiver: mpsc::Receiver<()>,
    renderer: R,
}

impl<R> App<R>
where
    R: Renderer,
{
    pub fn new(receiver: mpsc::Receiver<()>, mut renderer: R) -> Self {
        let state = State::default();
        renderer.render(&state);

        Self { receiver, renderer }
    }

    pub async fn run(&mut self) {
        while let Some(_) = self.receiver.recv().await {
            self.renderer.render(&State);
        }
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
        let expected_state = State::default();

        let (_, receiver) = mpsc::channel(1);

        let mut renderer_mock = MockRenderer::new();

        renderer_mock
            .expect_render()
            .times(1)
            .with(eq(expected_state))
            .return_const(());

        let _ = App::new(receiver, renderer_mock);
    }

    #[tokio::test]
    async fn render_frame_on_update() {
        let expected_state = State::default();

        let (sender, receiver) = mpsc::channel(1);

        let mut renderer_mock = MockRenderer::new();

        renderer_mock
            .expect_render()
            .times(2)
            .with(eq(expected_state))
            .return_const(());

        let mut tested_app = App::new(receiver, renderer_mock);

        task::spawn(async move {
            sender.send(()).await.unwrap();
        });

        tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await;
        })
        .await
        .unwrap();
    }
}
