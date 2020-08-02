mod dice;
mod state;
mod ui;
mod utils;

use std::sync::mpsc;

fn main() {
    let state = state::build_state();

    let (tx, rx) = mpsc::channel::<ui::ControllerMessage>();
    let mut ui = ui::Ui::new(tx);

    ui.display_state(state);

    while ui.step() {
        while let Some(msg) = rx.try_iter().next() {
            match msg {
                ui::ControllerMessage::LogMessage(msg) => {
                    // TODO write to app state
                    ui.send(ui::UiMessage::Log(msg))
                }
                ui::ControllerMessage::AddNote(note) => {
                    // TODO write to app state
                    ui.send(ui::UiMessage::Log(note))
                }
            }
        }
    }
}
