use::anyhow::anyhow;
pub use crate::db_service;
use std::io::stdout;
pub use crate::editor;
use::ratatui::{DefaultTerminal, Frame};
use::crossterm::event::{KeyEvent, Event, KeyCode};
use ratatui::layout::{Constraint, Layout,Direction};
use ratatui::widgets::{Row, Table, TableState,Block,Wrap, Clear, Paragraph};
use ratatui::style::{Style, Stylize};

use ratatui::backend::CrosstermBackend;
pub use crate::sound_service::{SoundControlls};

pub use crate::sound_service;
pub struct App {
    pub is_exit: bool, // Kontrolliert ob die App weiter laufen soll -> Wenn false: App schließt
    pub song_list: Song_List, 
    pub player_controller: sound_service::PlayerController, //abstraction Layer to controll the
    pub editor: editor::Editor,                                //audio playback
    pub show_edit: bool,
     
    pub show_selected_mode: bool,
    pub terminal: DefaultTerminal,
    //pub current_edit:
}
impl App {
    pub fn new(p_parent: Parent) -> Result<Self,anyhow::Error>{
      Ok(Self{
        is_exit : false,
        song_list : Song_List::new(p_parent)?, 
        player_controller: sound_service::PlayerController::new(),
        editor : editor::Editor::new(editor::EditorType::Default)?,
        show_edit: false,

        show_selected_mode: false, 
        terminal: (ratatui::DefaultTerminal::new(CrosstermBackend::new(stdout())).unwrap()),

      })
    }

        
    // Basiclly main methode 
    // hauptsächlich ratatui boilerplate
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), anyhow::Error> {    
        //---terminal Prep
        terminal.clear()?;
        crossterm::terminal::enable_raw_mode()?;
        //---

        //---main loop renderd frame und checkt keyEvent
        while self.is_exit == false{
            match terminal.draw(|frame|self.render(frame)){
                Ok(_t) => (),
                Err(_e) => self.is_exit = true,
                };
            if crossterm::event::poll(std::time::Duration::from_millis(100))?{
                
                match crossterm::event::read()?{
                    Event::Key(e) => match self.handle_key_press(e){
                        Ok(_t) => (),
                        Err(e) => {
                            eprintln!("{e}");
                            self.is_exit = true},
                    },
                    _ => (), 
                }
            }
        }
        crossterm::terminal::disable_raw_mode()?;
        terminal.clear();

        
        //---
        
        //--- Terminal cleanup
        Ok(())
        //---
    }


    // Methode ist dafür zuständig den Frame zu rendern
    pub fn render(&mut self,frame: &mut Frame)  {
        //---create first layout
        //splits screen verticly in half
        let layout_1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(frame.area());
        //---
        
        //--- top half of screen is split Horizontal
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .margin(1)
            
            .split(layout_1[0]);
       //---

        //--- bottom half is split Horizontal
        let bottom_layout= Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .horizontal_margin(1)
            .vertical_margin(1)
            .split(layout_1[1]);
        //----
        //---- popup for creating playlist

        //--- popup for editing 
        if self.show_edit{
            // creates a Block in wich the editor is rendered
            //
            let popup_block = Block::bordered().title("ligma");
            let centered_area = frame.area().centered(Constraint::Percentage(60),Constraint::Percentage(20));
            frame.render_widget(Clear, centered_area);
            //----
            //gets the song wich is edited
                   //----
            let outer_edit = Block::bordered().title(self.editor.editor_head.clone());
            //create Editor

            let paragraph = Paragraph::new("Lorem ipsum").block(popup_block);
            let widths = [
                Constraint::Percentage(10),
                Constraint::Percentage(90),
            ];
            let mut rows : Vec<Row> = vec![];
                for item in &self.editor.items{
                   rows.push(Row::new([item.0.clone(),item.1.clone()])) 
                }


            let edit_table = Table::new(rows,widths)
                .block(outer_edit)
                .cell_highlight_style(Style::new().reversed());



            frame.render_stateful_widget(edit_table, centered_area,&mut self.editor.table_state);
            



        }
        // ---------Create Table wich show selectable songs/albumns
        //--- Block around main Block
        let outer_main = Block::bordered()
            .title("Songs");

        let widths = [ // Widths for table rows
            Constraint::Percentage(70),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ];
        let mut rows:Vec<Row> = vec![];
        for val in self.song_list.rows.clone(){
            rows.push(Row::new(val));
        }
        let table = Table::new(rows,widths)
            .row_highlight_style(Style::new().reversed())
            .block(outer_main)
            .highlight_symbol("\u{f040a}");
        frame.render_stateful_widget(table,top_layout[0],&mut self.song_list.table_state);

        //---

        // -------currently playing widget
        let mut song_id: i64 = 0;
        // the func returns a Result<RwLock<Option<&str>>>
        // check if the result is Ok
        if let Ok(guard) = self.player_controller.current_song.try_read(){
            if let Some(i) = *guard{
               song_id = i.clone();
            }
        }
        let name: String =  db_service::get_song_info(&song_id).expect("Error in render in tui service").song_name;
          
        let block = Block::bordered().title("Currently Playing");
        let paragraph = ratatui::widgets::Paragraph::new(format!(
            "{} \u{f040c}",name))

            .block(block)
            .wrap(Wrap { trim: true })
            .centered();
        frame.render_widget(paragraph, bottom_layout[0]);
       //----------------- 
        
        //------ Next Songs widget
        let widths = [ // Widths for table rows
            Constraint::Percentage(70),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ];
        //  Prepare Struct
        let mut rows:Vec<Row> = vec![];
        // Terrible Performance
        if let Ok(guard) = self.player_controller.next_songs.try_read(){
            match guard.as_ref(){
                Some(i) => {
                    for val in i.clone(){
                        rows.push(Row::new(vec![db_service::get_song_info(&val).expect("in render in tui_service 123").song_name]));
                    }
                    let outer_right = Block::bordered()
                        .title("NEXT SONG");

                    let table = Table::new(rows,widths)
                        .block(outer_right);
                    frame.render_widget(table,top_layout[1]);
                },
                None => (),
            }
        }
        //---

 


        //---- Outer Block
        let outer_block = Block::bordered()
        .title("Music Player");
        frame.render_widget(outer_block, frame.area());

    }
    

    
    pub fn handle_key_press(&mut self,key_event: KeyEvent) -> Result<(), anyhow::Error>{
        if self.show_edit{ 
            // get selected Song
            
            //create Editor
                match key_event.code{
                    KeyCode::Char('q') => self.exit_edit()?,
                    x => self.editor.handle_keys(x)?,
                
        
                }

        }else if self.show_selected_mode{
             
        }
        else {
                match key_event.code{
                    KeyCode::Char('D') => self.delete_current()?,
                    KeyCode::Char('a') => self.playlist_adder()?,
                    KeyCode::Char('q') => self.q_pressed().expect(""),
                    KeyCode::Char('j') => self.song_list.table_state.select_next(),
                    KeyCode::Char('k') => self.song_list.table_state.select_previous(),
                    KeyCode::Char('e') => self.e_pressed().expect(""),
                    KeyCode::Char(' ') => self.player_controller.controll(SoundControlls::PlayPause).expect(""),
                    KeyCode::Char('J') => self.move_song(1)?,
                    KeyCode::Char('K') => self.move_song(-1)?,
                    KeyCode::Enter => self.enter_pressed().expect(""),
                    KeyCode::Char('l') => self.player_controller.controll(SoundControlls::SkipSong).expect(""),
                    KeyCode::Char('i') => self.i_pressed()?,
                    KeyCode::Char('c') =>  self.create_playlist()?,

                    _ => eprintln!("Error"),
                }
            }
        Ok(())
    
    }

    pub fn move_song(&mut self,increment:i64) -> Result<(),anyhow::Error>{
        // get song -> update song pos
        let selected = self.song_list.table_state.selected().expect("");
        let name = &self.song_list.rows[selected][0];
        let song = db_service::get_song_by_name(name).expect("in i pressed()");
        match &self.song_list.parent{
           Parent::playlist_name(name) =>{
                let playlist = db_service::get_playlist_by_name(&name)?;
                db_service::mov_song_up(&song,playlist.list_id,increment)?;
                self.song_list = Song_List::new(Parent::playlist_name(name.to_string()))?;

                let offset = (increment.abs()) as usize;
                if increment < 0 {
                    if selected > offset{
                        self.song_list.table_state.select(Some(selected-offset));
                    }
                    else{
                        return Ok(())
                    }
    
                }
                else{
                 
                    self.song_list.table_state.select(Some(selected+offset));
                }
           },
           Parent::default => (),
        }
        Ok(())
        //db_service::mov_song_up(&song,4);
        



    }
    pub fn delete_current(&mut self) -> Result<(), anyhow::Error>{
         
        let selected = self.song_list.table_state.selected().expect("");
        match &self.song_list.parent{
            Parent::playlist_name(name) =>{
                let song_name = &self.song_list.rows[selected][0];
                let song = db_service::get_song_by_name(song_name).expect("in i pressed()");
                let playlist = db_service::get_playlist_by_name(name)?;
                db_service::remove_song_from_playlist(playlist.list_id, song.song_id)?;
            }
            Parent::default => {
                
                let list_name = &self.song_list.rows[selected][0];
                db_service::remove_playlist(list_name)?;
            }
            _=> (),
       }
       let parent = match &self.song_list.parent{
            Parent::playlist_name(name) => Parent::playlist_name(name.clone()),
            Parent::default => Parent::default,

        };
        self.song_list = Song_List::new(parent)?;
        self.song_list.table_state.select(Some(selected));


       Ok(())
    }
    pub fn exit_edit(&mut self) -> Result<(),anyhow::Error>{
        
        let selected = self.song_list.table_state.selected().expect("");
        match self.editor.save(){
            Ok(_) => self.show_edit = false,

            Err(e) => eprintln!("cant save current edit Error {}",e),
        }

        let parent = match &self.song_list.parent{
            Parent::playlist_name(name) => Parent::playlist_name(name.clone()),
            Parent::default => Parent::default,

        };
        self.song_list = Song_List::new(parent)?;
        self.song_list.table_state.select(Some(selected));

        self.show_edit = false;
        Ok(())
    }
    pub fn append_to_item(&mut self, c: char){
        match self.editor.table_state.selected_cell(){
            Some(tup) => {
                let i = tup.0;
                _ = self.editor.items[i].1.push(c);
            },
            None => (),
        }
    }
    pub fn back_space(&mut self){
        match self.editor.table_state.selected_cell(){
            Some(tup) => {
                let i = tup.0;
                _ = self.editor.items[i].1.pop();
            }
            None => (),
        };

    }
    pub fn create_playlist(&mut self) -> Result<(),anyhow::Error>{
        self.show_edit = true;
        self.editor = editor::Editor::new(editor::EditorType::NewPlaylist)?;
        return Ok(())
    }
    pub fn i_pressed(&mut self) -> Result<(),anyhow::Error>{
        self.show_edit = true;
        let selected = self.song_list.table_state.selected().expect("");
        match &self.song_list.parent{
            Parent::playlist_name(p_name) =>{
                let name = &self.song_list.rows[selected][0];
                let song = db_service::get_song_by_name(name).expect("in i pressed()");
                self.editor = editor::Editor::new(editor::EditorType::Song(song.song_id))?;
                self.editor.table_state.select_cell(Some((0,1)));
                return Ok(())


                
            },
            Parent::default => {
                let name = &self.song_list.rows[selected][0];
                let playlist = db_service::get_playlist_by_name(name)?;
                self.editor = editor::Editor::new(editor::EditorType::Playlist(playlist.list_id))?;
            },

        }


       Ok(())

    }
    pub fn playlist_adder(&mut self) -> Result<(),anyhow::Error>{
        self.show_edit = true;

        let selected = self.song_list.table_state.selected().expect("");
        let name = &self.song_list.rows[selected][0];
        eprintln!("name {}",name);
        let song = db_service::get_song_by_name(name).expect("in i pressed()");



        self.editor = editor::Editor::new(editor::EditorType::AddToPlaylist(song.song_id))?;
        Ok(())

    }

    pub fn q_pressed(&mut self) -> Result<(),anyhow::Error>{
        match self.song_list.parent{
            Parent::playlist_name(_) =>{

                self.song_list = Song_List::new(Parent::default).expect("");


            },
            Parent::default =>{
                self.is_exit = true;
            },

        }
        Ok(())
    }

    pub fn enter_pressed(&mut self) -> Result<(),anyhow::Error>{
        match self.song_list.parent{
            Parent::playlist_name(_) =>{
                let selected = self.song_list.table_state.selected().expect("here");
                let name = &self.song_list.rows[selected][0];
                let song = db_service::get_song_by_name(name)?;
                self.player_controller.controll(sound_service::SoundControlls::AddSong(song))?;
            },
            Parent::default =>{
                let r = match self.song_list.table_state.selected(){
                    Some(t) => t,
                    None => return Err (anyhow!("Error in db::db_service::handle_key_press, can not acces Elemtent ")),
                };
                let name = &self.song_list.rows[r][0];
                let songs = db_service::get_songs_from_playlist(name)?;
                let _ = self.player_controller.controll(SoundControlls::AddPlaylist(songs));
            }
        }
        Ok(())
    }

    pub fn e_pressed(&mut self) -> Result<(),anyhow::Error>{
        match self.song_list.parent{
            Parent::playlist_name(_) =>{
                let selected = self.song_list.table_state.selected().expect("here");
                let name = &self.song_list.rows[selected][0];
                let song = db_service::get_song_by_name(name)?;
                self.player_controller.controll(sound_service::SoundControlls::AddSong(song))?;
            },
            Parent::default =>{
                let r = match self.song_list.table_state.selected(){
                    Some(t) => t,
                    None => return Err (anyhow!("Error in db::db_service::handle_key_press, can not acces Elemtent ")),
                };
                let name = &self.song_list.rows[r][0];
                let parent = Parent::playlist_name(db_service::get_playlist_by_name(name)?.list_name);
                self.song_list = Song_List::new(parent)?; 
            }
        }
        Ok(())
    }
        

}

impl Drop for App {
    fn drop(&mut self){
        crossterm::terminal::disable_raw_mode().expect("error while trying to restore Terminal after dropping App");


    }

}

    



pub enum Parent{
    playlist_name(String),
    default,
}
pub struct Song_List{ //stores information about current albums/Songs that are displayed in the
                      //main table
    pub  rows: Vec<Vec<String>> , //vec wich stores All displayed items
    pub parent: Parent, 
    pub table_state: TableState, //stores courser position
}
impl Song_List{
    pub fn new(parent:Parent) -> Result<Self,anyhow::Error>{
        match parent{
            Parent::default =>{
                let v_playlists = db_service::get_all_playlist()?;
                let mut rows: Vec<Vec<String>> = vec![] ;
                for val in v_playlists{
                    rows.push(vec![val.list_name]);
                }

                Ok(Self{

                    rows: rows,
                    parent: parent,
                    table_state: TableState::new().with_selected(Some(0)),
                })
            }
            Parent::playlist_name(t)=> { 
                let mut rows: Vec<Vec<String>> = vec![];
                for val in db_service::get_songs_from_playlist(&t)?{
                    rows.push(vec![val.song_name]) 
                }

                Ok(Self{
                    rows:rows,
                    parent:Parent::playlist_name(t),
                    table_state: TableState::new().with_selected(Some(0)),
                })

            }

        }
    }
            
                
        
}
