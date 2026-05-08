use::anyhow::anyhow;
pub use crate::db_service;
use::ratatui::{DefaultTerminal, Frame};
use::crossterm::event::{KeyEvent, Event, KeyCode};
use ratatui::layout::{Constraint, Layout,Direction};
use ratatui::widgets::{Row, Table, TableState,Block,Wrap, Clear, Paragraph};
use ratatui::style::{Style, Stylize};
pub use crate::sound_service::{SoundControlls};

pub use crate::sound_service;
pub struct App{
    pub is_exit: bool, // Kontrolliert ob die App weiter laufen soll -> Wenn false: App schließt
    pub song_list: Song_List, 
    pub player_controller: sound_service::PlayerController, //abstraction Layer to controll the
                                                            //audio playback
    pub show_edit: bool,
}
impl App {
    pub fn new(p_parent: Parent) -> Result<Self,anyhow::Error>{
      Ok(Self{
        is_exit : false,
        song_list : Song_List::new(p_parent)?, 
        player_controller: sound_service::PlayerController::new(),
        show_edit: false,
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
            match terminal.draw(|terminal|self.render(terminal)){
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
        //---
        
        //--- Terminal cleanup
        crossterm::terminal::disable_raw_mode()?;
        terminal.clear();
        Ok(())
        //---
    }


    // Methode ist dafür zuständig den Frame zu rendern
    pub fn render(&mut self,frame: &mut Frame){
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
        //--- popup for editing 
        if self.show_edit{
            let popup_block = Block::bordered().title("edit");
            let centered_area = frame.area().centered(Constraint::Percentage(60),Constraint::Percentage(20));
            frame.render_widget(Clear, centered_area);
            let paragraph = Paragraph::new("Lorem ipsum").block(popup_block);
            frame.render_widget(paragraph, centered_area);



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
        let name: String =  db_service::get_song_info(&song_id).expect("").song_name;
          
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

        //--- Block around right side
                }

    
    pub fn handle_key_press(&mut self,key_event: KeyEvent) -> Result<(), anyhow::Error>{
        match key_event.code{
            KeyCode::Char('q') => self.q_pressed()?,
            KeyCode::Char('j') => self.song_list.table_state.select_next(),
            KeyCode::Char('k') => self.song_list.table_state.select_previous(),
            KeyCode::Char('e') => self.e_pressed()?,
            KeyCode::Char(' ') => self.player_controller.controll(SoundControlls::PlayPause)?,
            KeyCode::Enter => self.enter_pressed()?,
            KeyCode::Char('l') => self.player_controller.controll(SoundControlls::SkipSong)?,
            KeyCode::Char('i') => self.show_edit = !self.show_edit,

            _ => eprintln!("{:?}",self.song_list.table_state),
        }
        Ok(())
    }

    pub fn q_pressed(&mut self) -> Result<(),anyhow::Error>{
        match self.song_list.parent{
            Parent::playlist_name(_) =>{
                self.song_list = Song_List::new(Parent::default)?;

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
