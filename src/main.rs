use std::thread::sleep;

use actor_model_chat::renderer::{terminal_renderer::TerminalRenderer, Renderer, State};

fn main() {
    let mut renderer = TerminalRenderer::new(std::io::stdout()).unwrap();

    let state = State::default();

    renderer.render(&state).unwrap();

    sleep(std::time::Duration::from_secs(2));
}
