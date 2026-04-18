use::std::io;
use::std::io::ErrorKind;
pub use crate::db_service;
use::ratatui::{DefaultTerminal, Frame};
use::crossterm::event::{KeyEvent, Event, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect,Direction};
use ratatui::widgets::{Row, Table, TableState,Block};
use ratatui::style::{Style, Stylize};
pub struct App{
    pub is_exit: bool, //When true App exit
    pub song_list: Song_List,
}
impl App {
    pub fn new(p_parent: Parent) -> Result<Self,std::io::Error>{
      Ok(Self{
        is_exit : false,
        song_list : Song_List::new(p_parent)?,
      })
      
    }


    // Basiclly main methode 
    // hauptsächlich ratatui boilerplate
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), std::io::Error> {    
        terminal.clear();
        crossterm::terminal::enable_raw_mode()?;
        // main loop renderd frame und checkt keyEvent
        while self.is_exit == false{
            match terminal.draw(|terminal|self.render(terminal)){
                Ok(_t) => (),
                Err(_e) => self.is_exit = true,
            };
            match crossterm::event::read()?{
                Event::Key(e) => match self.handle_key_press(e){
                    Ok(_t) => (),
                    Err(_e) => self.is_exit = true,
            
                },
                _ => (),
            }
        }
        crossterm::terminal::disable_raw_mode()?;
        terminal.clear();
        Ok(())
    }


    // Methode ist dafür zuständig den Frame zu rendern
    pub fn render(&mut self,frame: &mut Frame){

        // create first layout
        // f
        let layout_1 = Layout::default()
        
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .margin(2)
            .split(frame.area());


        // ---------Create Table 
        let widths = [ // Widths for table rows
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ];
        //  Prepare Struct
        let mut rows:Vec<Row> = vec![];
        // Terrible Performance
        for val in self.song_list.rows.clone(){//&vec<String>
            rows.push(Row::new(val));
        }
        let table = Table::new(rows,widths)
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>");
        frame.render_stateful_widget(table,layout_1[0],&mut self.song_list.table_state);
       //----------------- 
        
        //---- Outer Block
        let outer_block = Block::bordered().title("Music Player");
        frame.render_widget(outer_block, frame.area());
        
//        std::thread::sleep(std::time::Duration::from_secs(3));
    }



    

    pub fn handle_key_press(&mut self,key_event: KeyEvent) -> Result<(), std::io::Error>{
        match key_event.code{
            KeyCode::Char('q') => self.is_exit = true,
            KeyCode::Char('j') => self.song_list.table_state.select_next(),
            KeyCode::Char('k') => self.song_list.table_state.select_previous(),
            KeyCode::Char('e') => { 
                let r = match self.song_list.table_state.selected(){
                    Some(t) => t,
                    None => return Err (std::io::Error::new(ErrorKind::Other, "failed to read selected row")),
                };
                let name = &self.song_list.rows[r][0];
                let parent = Parent::playlist_name(db_service::get_playlist_by_name(name)?.list_name);
                self.song_list = Song_List::new(parent)?; 
            }
            
            _ => (),
        }
        Ok(())

    }
        

}


pub enum Parent{
    playlist_name(String),
    default,
}
pub struct Song_List{
    pub  rows: Vec<Vec<String>> ,
    pub parent: Parent,
    pub table_state: TableState, 
}
impl Song_List{
    pub fn new(parent:Parent) -> Result<Self,std::io::Error>{
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
