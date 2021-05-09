use rust_mqtt_chat::{
    app::App, events_reader::terminal::CrosstermEventsHandler, network::setup_network,
    renderer::terminal_renderer::TerminalRenderer,
};

use anyhow::Result;
use structopt::StructOpt;

const TOPIC_PREFIX: &str = "df9ff5c8-c030-4e4a-8bae-a415565febd7";

#[derive(StructOpt)]
#[structopt(
    name = "rust mqtt chat",
    about = "Payload encrypted chat over mqtt (written in rust)"
)]
struct Opt {
    /// Url to mqtt server
    #[structopt(short, long, env)]
    server: String,

    /// Name of chat room to connect to
    #[structopt(short, long, env)]
    room: String,

    /// Rooms password
    #[structopt(short, long, env)]
    password: String,

    /// User name
    #[structopt(short, long, env)]
    user: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let (events_publisher, network_events) = setup_network(
        opt.server,
        TOPIC_PREFIX,
        opt.room,
        opt.user.clone(),
        &opt.password,
    )
    .await?;
    let terminal_events = CrosstermEventsHandler::new();
    let renderer = TerminalRenderer::new(std::io::stdout())?;

    let mut app = App::new(
        opt.user,
        network_events,
        terminal_events,
        renderer,
        events_publisher,
    )
    .await;

    app.run().await?;

    Ok(())
}
