//use::std::{fs,io};
use std::path::Path;
use::crossterm::event;
use::ratatui;
use std::io::stdout;
use std::fs::File;
use std::io::prelude::*;
use ratatui::backend::CrosstermBackend;
use std::fs::OpenOptions;
pub mod db_service;
pub mod file_service;
pub mod tui_service;
fn main() {
let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/home/tom/programmieren/Music/log.txt")
        .unwrap();




let musik_path = Path::new("/home/tom/Musik");
db_service::init();
file_service::update_songs(&musik_path);
db_service::get_song_count();
let mut app = tui_service::App::new(tui_service::Parent::default).unwrap();
match app.run(&mut ratatui::DefaultTerminal::new(CrosstermBackend::new(stdout())).unwrap()){
    Ok(())=> (),
    Err(e) => panic!("Error: '{e}'") //file.write_all(format!("Error in Main: '{}'",e).as_bytes()).expect(""), 
};

}


