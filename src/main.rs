#![feature(trait_alias)]

mod dice;
mod dice_ui;
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

    /*
    let mut siv = cursive::default();
    siv.load_toml(include_str!("style.toml")).unwrap();

    let mut s = State::default();
    let mut characters = vec![
        Character::new("Tobias Wibble", "24/24"),
        Character::new("Patrik Arvidsson", "24/24"),
        Character::new("Goblin #1", "24/24"),
        Character::new("Alexander Arvidsson", "24/24"),
        Character::new("Goblin #2", "24/24"),
        Character::new("Christoffer Arvidsson", "24/24"),
        Character::new("Goblin #3", "24/24"),
        Character::new("Goblin #4", "24/24"),
    ];
    characters[2].notes = Some("dazed".to_string());

    s.characters.extend(characters);

    let state = Rc::new(RefCell::new(s));

    let mut player_list = SelectView::<String>::new();
    draw_character_list(&mut player_list, &state.borrow());

    siv.add_global_callback('q', move |s| {
        s.quit();
    });

    let state_rc = state.clone();
    siv.add_global_callback('r', move |s| {
        show_roll_dice_dialog(s, state_rc.clone());
    });

    let state_rc = state.clone();
    siv.add_global_callback('N', move |s| {
        show_notes_dialog(s, state_rc.clone());
    });

    siv.add_global_callback('o', move |s| {
        s.call_on_name(
            "log_scroll",
            |view: &mut ScrollView<NamedView<ListView>>| {
                let current_offset = view.content_viewport().top_left();
                let new_offset = current_offset.y + 12;
                view.set_offset(XY {
                    x: 0,
                    y: new_offset,
                });
            },
        );
    });

    siv.add_global_callback('i', move |s| {
        s.call_on_name(
            "log_scroll",
            |view: &mut ScrollView<NamedView<ListView>>| {
                if !view.is_at_top() {
                    let current_offset = view.content_viewport().top_left();
                    let new_offset = if current_offset.y <= 12 {
                        0
                    } else {
                        current_offset.y - 12
                    };
                    view.set_offset(XY {
                        x: 0,
                        y: new_offset,
                    });
                }
            },
        );
    });

    let state_rc = state.clone();
    siv.add_global_callback('n', move |s| {
        let mut state = state_rc.borrow_mut();
        s.call_on_name("player_list", |view: &mut SelectView<String>| {
            state.selected_index = (state.selected_index + 1) % state.characters.len();
            view.clear();
            draw_character_list(view, &state);
        });

        let curr_name = state.characters[state.selected_index].name.clone();
        log(s, &mut state, format!("Turn: {}", curr_name));
    });

    let state_rc = state.clone();
    siv.add_global_callback('p', move |s| {
        let mut state = state_rc.borrow_mut();
        s.call_on_name("player_list", |view: &mut SelectView<String>| {
            state.selected_index = if state.selected_index == 0 {
                state.characters.len() - 1
            } else {
                (state.selected_index - 1) % state.characters.len()
            };
            view.clear();
            draw_character_list(view, &state);
        });

        let curr_name = state.characters[state.selected_index].name.clone();
        log(s, &mut state, format!("Turn: {}", curr_name));
    });

    let encounter_list = LinearLayout::vertical().child(TextView::new("> Goblin ambush"));

    let encounter_panel =
        Panel::new(encounter_list.resized(SizeConstraint::Full, SizeConstraint::Full))
            .title("Encounters");

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
    let root = LinearLayout::horizontal().child(panel1).child(panel2);

    siv.add_layer(root);

    siv.run();
    */
}

/*
fn show_input_dialog<F, R>(s: &mut Cursive, title: &str, callback: F)
where
    F: 'static + Clone + Fn(&mut Cursive, &str) -> R,
{
    let cb1 = callback.clone();
    let cb2 = callback.clone();
    let dialog = Dialog::new()
        .title(title)
        .padding(Margins::lrtb(1, 1, 1, 0))
        .content(
            EditView::new()
                .on_submit(move |s: &mut Cursive, name: &str| {
                    cb1(s, name);
                    s.pop_layer();
                })
                .filler(" ")
                .style(ColorStyle {
                    front: ColorType::Palette(PaletteColor::Highlight),
                    back: ColorType::Palette(PaletteColor::Primary),
                })
                .with_name("input_field")
                .fixed_width(20),
        )
        .button("Ok", move |s| {
            let input = s
                .call_on_name("input_field", |view: &mut EditView| view.get_content())
                .unwrap();

            cb2(s, &input);
            s.pop_layer();
        });
    s.add_layer(dialog);
}

fn draw_log(view: &mut ListView, state: &State) {
    view.clear();
    for msg in &state.log_messages {
        view.add_child("", TextView::new(msg));
    }
}

fn log<S>(s: &mut Cursive, state: &mut State, msg: S)
where
    S: ToString,
{
    state.log_messages.push(msg.to_string());
    s.call_on_name("log", |view: &mut ListView| {
        draw_log(view, &state);
    });
}

fn add_note<S>(s: &mut Cursive, state: &mut State, note: S)
where
    S: ToString,
{
    state.characters[state.selected_index].notes = Some(note.to_string());
    s.call_on_name("player_list", |view: &mut SelectView<String>| {
        view.clear();
        draw_character_list(view, &state);
    });
}

*/
