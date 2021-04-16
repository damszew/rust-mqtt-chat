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
                    self.state.cursor += 1;
                }
                AppEvent::Accept => {
                    let msg = self.state.input_message.drain(..).collect();
                    self.state.cursor = 0;
                    self.state.messages.push(msg);
                }
                AppEvent::RemoveLast => {
                    self.state.input_message.pop();
                    if self.state.cursor > 0 {
                        self.state.cursor -= 1;
                    }
                }
                AppEvent::CursorStart => {
                    self.state.cursor = 0;
                }
                AppEvent::CursorEnd => {
                    self.state.cursor = self.state.input_message.len();
                }
                AppEvent::CursorRight => {
                    if self.state.cursor < self.state.input_message.len() {
                        self.state.cursor += 1;
                    }
                }
                AppEvent::CursorLeft => {
                    if self.state.cursor > 0 {
                        self.state.cursor -= 1;
                    }
                }
                _ => todo!(),
            }
            self.renderer.render(&self.state)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod should {
    use std::time::Duration;

    use mockall::{predicate::*, Sequence};
    use test_case::test_case;
    use tokio::{sync::mpsc, task};

    use super::*;

    use crate::renderer::{MockRenderer, State};

    #[test]
    fn render_frame_on_startup() {
        let renderer_mock = setup_rendered_mock();

        let (_, receiver) = mpsc::channel(1);
        let _ = App::new(receiver, renderer_mock);
    }

    #[test_case(
        vec![AppEvent::Character('c')],
        vec![State {
            input_message: "c".into(),
            cursor: 1,
            ..Default::default()
        }]
        ; "single character")]
    #[test_case(
        vec![
            AppEvent::Character('m'),
            AppEvent::Character('e'),
            AppEvent::Accept,
        ],
        vec![
            State {
                input_message: "m".into(),
                cursor: 1,
                ..Default::default()
            },
            State {
                input_message: "me".into(),
                cursor: 2,
                ..Default::default()
            },
            State {
                input_message: "".into(),
                cursor: 0,
                messages: vec!["me".into()],
                ..Default::default()
            },
        ]
        ; "on accept push input to messages")]
    #[test_case(
        vec![
            AppEvent::Character('m'),
            AppEvent::Character('e'),
            AppEvent::RemoveLast,
        ],
        vec![
            State {
                input_message: "m".into(),
                cursor: 1,
                ..Default::default()
            },
            State {
                input_message: "me".into(),
                cursor: 2,
                ..Default::default()
            },
            State {
                input_message: "m".into(),
                cursor: 1,
                ..Default::default()
            },
        ]
        ; "remove last character")]
    #[test_case(
            vec![
                AppEvent::RemoveLast,
            ],
            vec![
                State {
                    input_message: "".into(),
                    cursor: 0,
                    ..Default::default()
                },
            ]
            ; "remove on empty buffer")]
    #[tokio::test]
    async fn render_frame_on_update(events: Vec<AppEvent>, expected_states: Vec<State>) {
        let mut seq = Sequence::new();
        let mut renderer_mock = setup_rendered_mock();
        for s in expected_states {
            renderer_mock
                .expect_render()
                .times(1)
                .with(eq(s))
                .in_sequence(&mut seq)
                .returning(|_| Ok(()));
        }

        let (sender, receiver) = mpsc::channel(1);
        let mut tested_app = App::new(receiver, renderer_mock);

        task::spawn(async move {
            for event in events {
                sender.send(event).await.unwrap();
            }
        });

        tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await.unwrap();
        })
        .await
        .unwrap();
    }

    #[test_case(
        vec![AppEvent::CursorLeft],
        vec![16]
        ; "single left")]
    #[test_case(
        vec![AppEvent::CursorStart, AppEvent::CursorRight],
        vec![0, 1]
        ; "to start and one right")]
    #[test_case(
        vec![AppEvent::CursorStart, AppEvent::CursorLeft],
        vec![0, 0]
        ; "cursor pos can not be negative")]
    #[test_case(
        vec![AppEvent::CursorStart, AppEvent::CursorEnd, AppEvent::CursorRight],
        vec![0, 17, 17]
        ; "do not exceed input message")]
    #[tokio::test]
    async fn move_cursor(events: Vec<AppEvent>, cursor_positions: Vec<usize>) {
        let init_state = State {
            input_message: "Some long message".into(),
            cursor: 17,
            ..Default::default()
        };

        let mut seq = Sequence::new();
        let mut renderer_mock = setup_rendered_mock();
        for s in cursor_positions {
            let mut state = init_state.clone();
            state.cursor = s;

            renderer_mock
                .expect_render()
                .times(1)
                .with(eq(state.clone()))
                .in_sequence(&mut seq)
                .returning(|_| Ok(()));
        }

        let (sender, receiver) = mpsc::channel(1);
        let mut tested_app = App::new(receiver, renderer_mock);
        tested_app.state = init_state;

        task::spawn(async move {
            for event in events {
                sender.send(event).await.unwrap();
            }
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
