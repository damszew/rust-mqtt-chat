mod network_events_handler;
mod terminal_events_handler;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::try_join;

use network_events_handler::NetworkEventsHandler;
use terminal_events_handler::TerminalEventsHandler;

use crate::{
    events_reader::EventsReader,
    network::NetworkEvent,
    renderer::{Renderer, State},
    AppEvent,
};

pub struct App<R, NE, TE>
where
    R: Renderer,
    NE: EventsReader<Message = NetworkEvent>,
    TE: EventsReader<Message = AppEvent>,
{
    network_events: NE,
    terminal_events: TE,
    renderer: R,
    state: Arc<Mutex<State>>,
}

impl<R, NE, TE> App<R, NE, TE>
where
    R: Renderer + Send,
    NE: EventsReader<Message = NetworkEvent> + Send,
    TE: EventsReader<Message = AppEvent> + Send,
{
    pub async fn new(mut network_events: NE, mut terminal_events: TE, renderer: R) -> Self {
        let state = Arc::new(Mutex::new(State::default()));

        let ne_handler = NetworkEventsHandler::new(state.clone());
        network_events
            .subscribe(move |message| {
                ne_handler.handle(message);
            })
            .await;

        let te_handler = TerminalEventsHandler::new(state.clone());
        terminal_events
            .subscribe(move |message| {
                te_handler.handle(message);
            })
            .await;

        Self {
            network_events,
            terminal_events,
            renderer,
            state,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let network_loop = self.network_events.run();
        let terminal_loop = self.terminal_events.run();

        let rendering_state = self.state.clone();
        let renderer = &mut self.renderer;
        let rendering_loop = async {
            loop {
                let state = rendering_state.lock().unwrap();
                if state.quit {
                    return Ok(());
                }
                renderer.render(&state)?;
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        };

        let result = try_join!(network_loop, terminal_loop, rendering_loop);

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod should {
    use std::time::Duration;

    use super::*;

    use crate::renderer::MockRenderer;

    #[tokio::test]
    async fn subscribe_on_network_events() {
        let mut renderer_mock = MockRenderer::new();
        renderer_mock.expect_render().returning(|_| Ok(()));

        let mut network_events_mock = MockNetworkEventsReaderMock::new();
        network_events_mock
            .expect_subscribe()
            .once()
            .return_const(());

        let mut terminal_events_mock = MockTerminalEventsReaderMock::new();
        terminal_events_mock.expect_subscribe().return_const(());

        let _ = App::new(network_events_mock, terminal_events_mock, renderer_mock).await;
    }

    #[tokio::test]
    async fn subscribe_on_terminal_events() {
        let mut renderer_mock = MockRenderer::new();
        renderer_mock.expect_render().returning(|_| Ok(()));

        let mut network_events_mock = MockNetworkEventsReaderMock::new();
        network_events_mock.expect_subscribe().return_const(());

        let mut terminal_events_mock = MockTerminalEventsReaderMock::new();
        terminal_events_mock
            .expect_subscribe()
            .once()
            .return_const(());

        let _ = App::new(network_events_mock, terminal_events_mock, renderer_mock).await;
    }

    #[tokio::test]
    async fn start_loop_receiving_terminal_events() {
        let mut renderer_mock = MockRenderer::new();
        renderer_mock.expect_render().returning(|_| Ok(()));

        let mut network_events_mock = MockNetworkEventsReaderMock::new();
        network_events_mock.expect_subscribe().return_const(());
        network_events_mock.expect_run().returning(|| Ok(()));

        let mut terminal_events_mock = MockTerminalEventsReaderMock::new();
        terminal_events_mock
            .expect_subscribe()
            .once()
            .return_const(());
        terminal_events_mock
            .expect_run()
            .once()
            .returning(|| Ok(()));

        let mut tested_app =
            App::new(network_events_mock, terminal_events_mock, renderer_mock).await;

        let _ = tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn start_loop_receiving_network_events() {
        let mut renderer_mock = MockRenderer::new();
        renderer_mock.expect_render().returning(|_| Ok(()));

        let mut network_events_mock = MockNetworkEventsReaderMock::new();
        network_events_mock
            .expect_subscribe()
            .once()
            .return_const(());
        network_events_mock.expect_run().once().returning(|| Ok(()));

        let mut terminal_events_mock = MockTerminalEventsReaderMock::new();
        terminal_events_mock.expect_subscribe().return_const(());
        terminal_events_mock.expect_run().returning(|| Ok(()));

        let mut tested_app =
            App::new(network_events_mock, terminal_events_mock, renderer_mock).await;

        let _ = tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn start_rendering_loop() {
        let mut renderer_mock = MockRenderer::new();
        renderer_mock
            .expect_render()
            .times(2..)
            .returning(|_| Ok(()));

        let mut network_events_mock = MockNetworkEventsReaderMock::new();
        network_events_mock.expect_subscribe().return_const(());
        network_events_mock.expect_run().once().returning(|| Ok(()));

        let mut terminal_events_mock = MockTerminalEventsReaderMock::new();
        terminal_events_mock.expect_subscribe().return_const(());
        terminal_events_mock.expect_run().returning(|| Ok(()));

        let mut tested_app =
            App::new(network_events_mock, terminal_events_mock, renderer_mock).await;

        let _ = tokio::time::timeout(Duration::from_millis(500), async move {
            tested_app.run().await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn stop_rendering_loop_on_quit() {
        let mut renderer_mock = MockRenderer::new();
        renderer_mock.expect_render().returning(|_| Ok(()));

        let mut network_events_mock = MockNetworkEventsReaderMock::new();
        network_events_mock.expect_subscribe().return_const(());
        network_events_mock.expect_run().once().returning(|| Ok(()));

        let mut terminal_events_mock = MockTerminalEventsReaderMock::new();
        terminal_events_mock.expect_subscribe().return_const(());
        terminal_events_mock.expect_run().returning(|| Ok(()));

        let mut tested_app =
            App::new(network_events_mock, terminal_events_mock, renderer_mock).await;

        tested_app.state.lock().unwrap().quit = true;

        let result = tokio::time::timeout(Duration::from_millis(100), async move {
            tested_app.run().await.unwrap();
        })
        .await;

        assert!(result.is_ok()); // App finished before timeout
    }

    mockall::mock! {
        NetworkEventsReaderMock {}

        #[async_trait::async_trait]
        impl EventsReader for NetworkEventsReaderMock {
            type Message = NetworkEvent;

            async fn subscribe<F>(&mut self, callback: F)
            where
                F: Fn(NetworkEvent) -> () + Send + 'static;

            async fn run(&mut self) -> Result<()>;
        }
    }

    mockall::mock! {
        TerminalEventsReaderMock {}

        #[async_trait::async_trait]
        impl EventsReader for TerminalEventsReaderMock {
            type Message = AppEvent;

            async fn subscribe<F>(&mut self, callback: F)
            where
                F: Fn(AppEvent) -> () + Send + 'static;

            async fn run(&mut self) -> Result<()>;
        }
    }
}
