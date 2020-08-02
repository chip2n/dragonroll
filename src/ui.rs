use crate::dice::ui::RollDiceDialog;
use crate::state;
use cursive::theme::*;
use cursive::traits::*;
use cursive::utils::span::SpannedString;
use cursive::view::*;
use cursive::views::*;
use cursive::Cursive;
use enumset::EnumSet;
use std::sync::mpsc;

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
    ui_tx: mpsc::Sender<UiMessage>,
    controller_tx: mpsc::Sender<ControllerMessage>,
}

pub enum UiMessage {
    Log(String),
}

pub enum ControllerMessage {
    LogMessage(String),
    AddNote(String),
}

impl Ui {
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>) -> Self {
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let mut ui = Ui {
            cursive: cursive::default(),
            ui_rx,
            ui_tx,
            controller_tx,
        };

        ui.cursive.load_toml(include_str!("style.toml")).unwrap();

        ui.cursive.add_global_callback('q', move |cursive| {
            cursive.quit();
        });

        let tx = ui.controller_tx.clone();
        ui.cursive.add_global_callback('r', move |cursive| {
            let dialog = RollDiceDialog::new(&tx);
            dialog.show(cursive);
        });

        let tx = ui.controller_tx.clone();
        ui.cursive.add_global_callback('N', move |cursive| {
            show_notes_dialog(cursive, &tx);
        });

        let root = build_root();
        ui.cursive.add_layer(root);
        ui
    }

    pub fn step(&mut self) -> bool {
        if !self.cursive.is_running() {
            return false;
        }

        while let Some(message) = self.ui_rx.try_iter().next() {
            match message {
                UiMessage::Log(msg) => self.add_log_msg(msg),
            }
        }

        self.cursive.refresh();
        self.cursive.step();

        true
    }

    pub fn send(&mut self, msg: UiMessage) {
        let _ = self.ui_tx.send(msg).unwrap();
    }

    pub fn display_state(&mut self, state: state::State) {
        let mut view = self
            .cursive
            .find_name::<SelectView<String>>("player_list")
            .unwrap();
        draw_character_list(&mut view, &state)
    }

    fn add_log_msg(&mut self, msg: String) {
        self.cursive.call_on_name("log", |view: &mut ListView| {
            view.add_child("", TextView::new(msg));
        });
    }
}

fn build_root() -> impl View {
    let encounter_list = LinearLayout::vertical().child(TextView::new("> Goblin ambush"));
    let encounter_panel =
        Panel::new(encounter_list.resized(SizeConstraint::Full, SizeConstraint::Full))
            .title("Encounters");

    let player_list = SelectView::<String>::new();
    let player_panel = Panel::new(
        player_list
            .with_name("player_list")
            .resized(SizeConstraint::Full, SizeConstraint::Full),
    )
    .title("Players");

    let panel1 = LinearLayout::vertical()
        .child(player_panel)
        .child(encounter_panel)
        .resized(SizeConstraint::Fixed(56), SizeConstraint::Full);

    let log_list = ListView::new().with_name("log").scrollable();

    let panel2 = Panel::new(log_list.with_name("log_scroll"))
        .title("Log")
        .resized(SizeConstraint::Full, SizeConstraint::Full);

    LinearLayout::horizontal().child(panel1).child(panel2)
}

fn show_notes_dialog(cursive: &mut Cursive, tx: &mpsc::Sender<ControllerMessage>) {
    let tx = tx.clone();
    let dialog = build_input_dialog("Notes", None, move |cursive, text| {
        tx.send(ControllerMessage::AddNote(text.to_string()))
            .unwrap();
        cursive.pop_layer();
    });
    cursive.add_layer(dialog);
}

pub fn build_input_dialog<F>(
    title: impl Into<String>,
    message: Option<String>,
    on_submit: F,
) -> impl View
where
    F: 'static + Clone + Fn(&mut Cursive, &str),
{
    let on_submit_clone = on_submit.clone();

    let input_field = EditView::new()
        .on_submit(move |cursive, input| on_submit_clone.clone()(cursive, input))
        .filler(" ")
        .style(ColorStyle {
            front: ColorType::Palette(PaletteColor::Highlight),
            back: ColorType::Palette(PaletteColor::Primary),
        })
        .with_name("input_field")
        .full_width();

    let mut content = LinearLayout::vertical().child(input_field);

    let msg = message.unwrap_or("".to_string());
    content.add_child(TextView::new(msg).with_name("input_msg"));

    Dialog::new()
        .title(title.into())
        .padding(Margins::lrtb(1, 1, 1, 0))
        .content(content)
        .button("Ok", move |cursive| {
            let input = cursive
                .call_on_name("input_field", |view: &mut EditView| view.get_content())
                .unwrap();

            on_submit(cursive, &input);
        })
        .max_width(40)
}

fn draw_character_list(view: &mut SelectView<String>, state: &state::State) {
    let longest_name = state
        .characters
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(0);
    for (i, c) in state.characters.iter().enumerate() {
        let name_length = c.name.chars().count();
        let padding = longest_name - name_length + 2;
        let dots = std::iter::repeat(".").take(padding).collect::<String>();
        let notes = c.notes.clone().unwrap_or("".to_string());
        let selection = if i == state.selected_index { ">" } else { " " };

        let mut span = SpannedString::styled(selection, Style::default());
        let name_span = SpannedString::styled(
            &c.name,
            Style {
                effects: EnumSet::only(Effect::Bold),
                color: Some(ColorStyle {
                    front: ColorType::Palette(PaletteColor::Primary),
                    back: ColorType::Palette(PaletteColor::Background),
                }),
            },
        );
        let rest_span =
            SpannedString::styled(format!("{}{} {}", dots, c.hp, notes), Style::default());
        span.append(name_span);
        span.append(rest_span);
        view.add_item(span, "".to_string());

        view.set_selection(state.selected_index);
    }
}
