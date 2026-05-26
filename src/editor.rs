use ratatui::widgets::{Table, TableState, Row};

use::crossterm::event::{KeyEvent, Event, KeyCode};
use ratatui::layout::{Constraint, Layout,Direction};
pub use crate::db_service;
use db_service::Song;
pub enum EditorType{
    Song(i64),
    Playlist(i64),
    Default,

}
pub struct Editor  {
    pub table_state: TableState,
    pub items: Vec<(String,String)>,
    pub editor_type: EditorType


}

impl  Editor{
    pub fn new(typ: EditorType) -> Result<Self,anyhow::Error>{
        match typ {
            EditorType::Song(p_song_id) =>{
                
                let song = db_service::get_song_info(&p_song_id)?;
                let items = vec![(String::from("name"),song.song_name)];
                eprintln!("{:?}",items);
                
                return Ok(Self{
                    items,
                    editor_type: EditorType::Song(p_song_id),

                    table_state: TableState::new().with_selected_cell(Some((0,1))),



                }) 


            }
            EditorType::Default => {
                let widths = [
                    Constraint::Percentage(100),
                ];
                let rows = [
                    Row::new(vec!["Default"])
                ];
                let table = Table::new(rows,widths);
                let mut items = vec![(String::from("name"),String::from("this is not supposed to happen"))];

            
                return Ok(Self{
                    items,
                    editor_type: EditorType::Default,
                    table_state: TableState::new().with_selected_cell(Some((0,1))),

            })}
            EditorType::Playlist(p_list_id) =>{
                let playlist = db_service::get_playlist_by_id(p_list_id)?;
                let items = vec![(String::from("name"),playlist.list_name)];
                return Ok(Self{
                    items,
                    editor_type: EditorType::Playlist(p_list_id),
                    table_state: TableState::new().with_selected_cell(Some((0,1)))
                })
            },
            _ => Err(anyhow::anyhow!("new is not implemented for EditorType::playlist")),
        

        }
            
    }
    pub fn save(&mut self) -> Result<(),anyhow::Error>{
        match self.editor_type{
            EditorType::Song(id) => {
                let mut song = db_service::get_song_info(&id)?;
                song.song_name = self.items[0].1.clone();
                db_service::update_song(&song)?;
                Ok(())
                

            },
            EditorType::Playlist(id) =>{
                let mut playlist = db_service::get_playlist_by_id(id)?;
                playlist.list_name = self.items[0].1.clone();
                db_service::update_playlist(&playlist)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("updating anything other than song not implemented")),
        }

    }
    pub fn temp(&mut self){
        self.table_state.select_next()

    }
    

    
        
    
    
    

}
