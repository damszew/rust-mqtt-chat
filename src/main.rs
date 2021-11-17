use rust_mqtt_chat::{
    chat_room::queue_chat_room::QueueChatRoom,
    crypto::magic_crypt::MagicCrypt,
    queue::{encrypted_queue::EncryptedQueue, mqtt::MqttQueue},
    tui::{components::main_view::MainView, terminal_driver::TerminalDriver},
};
use structopt::StructOpt;

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
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();

    let queue = MqttQueue::new(opt.server).await?;

    let crypto = MagicCrypt::new(&opt.password);
    let queue = EncryptedQueue::new(queue, crypto);

    let mut chat_room = QueueChatRoom::new(queue, opt.user, opt.room).await?;

    let ui = MainView::new(chat_room.clone());

    let mut driver = TerminalDriver::new(std::io::stdout())?;

    tokio::select! {
        r = chat_room.run() => {r?}
        r = driver.run(ui) => {r?}
    }

    Ok(())
}
