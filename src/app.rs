use std::sync::mpsc;

use crate::renderer::{Renderer, State};

struct App<R> {
    receiver: mpsc::Receiver<()>,
    renderer: R,
}

impl<R> App<R>
where
    R: Renderer,
{
    pub fn new(receiver: mpsc::Receiver<()>, mut renderer: R) -> Self {
        let state = State;
        renderer.render(&state);

        Self { receiver, renderer }
    }
}

#[cfg(test)]
mod should {
    use std::sync::mpsc;

    use mockall::predicate::*;

    use super::*;

    use crate::renderer::{MockRenderer, State};

    #[test]
    fn render_frame_on_startup() {
        let expected_state = State;

        let (_, receiver) = mpsc::channel();

        let mut renderer_mock = MockRenderer::new();

        renderer_mock
            .expect_render()
            .times(1)
            .with(eq(expected_state))
            .return_const(());

        let _ = App::new(receiver, renderer_mock);
    }

    #[test]
    fn render_frame_on_update() {}
}
