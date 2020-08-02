use crate::dice;
use crate::ui;
use crate::ui::ControllerMessage;
use cursive::views::*;
use cursive::Cursive;
use std::sync::mpsc::Sender;

pub struct RollDiceDialog {
    tx: Sender<ControllerMessage>,
}

impl RollDiceDialog {
    pub fn new(tx: &Sender<ControllerMessage>) -> Self {
        RollDiceDialog { tx: tx.clone() }
    }

    pub fn show(&self, cursive: &mut Cursive) {
        let tx = self.tx.clone();
        let dialog =
            ui::build_input_dialog("Roll dice", None, move |cursive, input| {
                let result = dice::eval(input);
                match result {
                    Some(num) => {
                        let msg = format!("Rolling: {} -> {:?}", input, num);
                        tx.send(ControllerMessage::LogMessage(msg)).unwrap();
                        cursive.pop_layer();
                    }
                    None => {
                        let mut view = cursive.find_name::<TextView>("input_msg").unwrap();
                        view.set_content("meh");
                    }
                };
            });
        cursive.add_layer(dialog);
    }
}
