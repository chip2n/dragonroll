#[derive(Default)]
pub struct State {
    pub characters: Vec<Character>,
    pub selected_index: usize,
    pub log_messages: Vec<String>,
}

pub struct Character {
    pub name: String,
    pub hp: String,
    pub notes: Option<String>,
}

impl Character {
    fn new(name: &str, hp: &str) -> Self {
        Character {
            name: name.to_string(),
            hp: hp.to_string(),
            notes: None,
        }
    }
}

pub fn build_state() -> State {
    let mut s = State::default();
    let mut characters = vec![
        Character::new("Player #1", "24/24"),
        Character::new("Player #2", "24/24"),
        Character::new("Monster #1", "24/24"),
        Character::new("Player #3", "24/24"),
        Character::new("Monster #2", "24/24"),
        Character::new("Player #4", "24/24"),
        Character::new("Monster #3", "24/24"),
        Character::new("Monster #4", "24/24"),
    ];
    characters[2].notes = Some("dazed".to_string());
    s.characters.extend(characters);
    s
}
