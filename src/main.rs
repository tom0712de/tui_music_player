//use::std::{fs,io};
use std::path::Path;
use::ratatui;
use std::io::stdout;
use std::io::prelude::*;
use ratatui::backend::CrosstermBackend;
use std::fs::OpenOptions;
pub mod db_service;
pub mod file_service;
pub mod tui_service;
pub mod sound_service;
pub mod editor;
pub mod Config;
fn main() {

// load config
let cfg: Config::Config = confy::load("Rusty-Music",None).expect("failed to load config");



//let musik_path = Path::new("/home/tom/Musik");//debug
let musik_path = Path::new(cfg.music_path.as_str()); //prod
match db_service::init(){
    Ok(()) => (),
    Err(e) => eprintln!("'Error while trying to call db_service::init in main: '{e}'")
};
match file_service::update_songs(&musik_path){
    Ok(_t) => (),
    Err(e) => eprintln!("Error while trying to update Songs from main: '{e}'"), 

};
let mut app = tui_service::App::new(
        tui_service::Parent::specific_attribute(tui_service::Filters::ShowAll,false)).unwrap();
//let sound = sound_service::new();
//sound.add_song_to_queue(db::service::get_song_info())
match app.run(&mut ratatui::DefaultTerminal::new(CrosstermBackend::new(stdout())).unwrap()){
    Ok(())=> (),
    Err(e) => eprintln!("Error: '{e}'"), //file.write_all(format!("Error in Main: '{}'",e).as_bytes()).expect(""), 
};

}


