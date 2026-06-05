use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub music_path: String,
    pub db_path: String,
    // hotkeys 
    pub exit: char,       // 'q'
    pub delete: char,     // 'D'
    pub add: char,        // 'a'
    pub down: char,       // 'j'
    pub up: char,         // 'k'
    pub edit: char,       // 'e'
    pub play_pause: char, // ' ' (space)
    pub move_down: char,  // 'J'
    pub move_up: char,    // 'K'
    pub skip: char,       // 'l'
    pub info: char,       // 'i'
    pub create: char,     // 'c


}

impl ::std::default::Default for Config {
    fn default() -> Self{
        Self{
            music_path: "music/".into(),
            db_path: "db.sqlite".into(),
            exit: 'q',
            delete: 'D',
            add: 'a',
            down: 'j',
            up: 'k',
            edit: 'e',
            play_pause: ' ',
            move_down: 'J',
            move_up: 'K',
            skip: 'l',
            info: 'i',
            create: 'c',
        }
    }
}
