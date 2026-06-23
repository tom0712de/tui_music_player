use ratatui::widgets::{Table, TableState, Row};

use::crossterm::event::{KeyEvent, Event, KeyCode};
use ratatui::layout::{Constraint, Layout,Direction};
pub use crate::db_service;
use db_service::Song;
#[derive(Default)]
pub enum EditorType{
    Song(i64),
    Playlist(i64),
    #[default]
    Default,
    NewPlaylist,
    AddToPlaylist(i64),

}
#[derive(Default)]
pub struct Editor  {
    pub table_state: TableState,
    pub items: Vec<(String,String)>,
    pub editor_type: EditorType,
    pub editor_head: String     

}

impl  Editor{
        
    pub fn new(typ: EditorType) -> Result<Self,anyhow::Error>{
        match typ {
            EditorType::Song(p_song_id) =>{
                
                let song = db_service::get_song_info(&p_song_id)?;
                let items = vec![(String::from("name"),song.song_name),
                    (String::from("genre"),song.genre),
                    (String::from("author"),song.author),
                    (String::from("date"),format!("{}",song.year))];
                eprintln!("{:?}",items);
                
                return Ok(Self{
                    items,
                    editor_type: EditorType::Song(p_song_id),
                    
                    table_state: TableState::new().with_selected_cell(Some((0,1))),
                    editor_head: String::from("Edit Song"),


                }) 


            }
            EditorType::AddToPlaylist(song_id)=>{
                let playlists = db_service::get_user_created_playlist()?;
                let mut items: Vec<(String,String)> = vec![(String::from(""),String::from(""))]; 
                for list in playlists{
                    items.push((String::from(""),list.list_name));

                }
                let song_name = db_service::get_song_info(&song_id)?.song_name;
                return Ok(Self{
                    items,
                    editor_type: EditorType::AddToPlaylist(song_id),
                    editor_head: format!("To which playlist should {} be added",song_name),
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
                    editor_head: String::from("why didnt i just use default trait"),

            })}
            EditorType::Playlist(p_list_id) =>{
                let playlist = db_service::get_playlist_by_id(p_list_id)?;
                let items = vec![(String::from("name"),playlist.list_name.clone())
                    ,((String::from("author"),playlist.author.clone())),(String::from("genre"),playlist.genre.clone())];
                return Ok(Self{
                    items,
                    editor_type: EditorType::Playlist(p_list_id),
                    table_state: TableState::new().with_selected_cell(Some((0,1))),
                    editor_head: String::from("Edit"),
                })
            },
            EditorType::NewPlaylist =>{
                let items = vec![(String::from("name"),String::from("_")),(String::from("Author"),String::from(""))];
                return Ok(Self{
                    items,
                    editor_type: EditorType::NewPlaylist,
                    table_state : TableState::new().with_selected_cell(Some((0,1))),
                    editor_head: String::from("create new Playlist"),

                })
            }
            _ => Err(anyhow::anyhow!("new is not implemented for EditorType::playlist")),
        

        }
            
    }
    pub fn save(&mut self) -> Result<(),anyhow::Error>{
        match self.editor_type{
            EditorType::Song(id) => {
                let mut song = db_service::get_song_info(&id)?;
                song.song_name = self.items[0].1.clone();
                song.genre = self.items[1].1.clone();
                song.author = self.items[2].1.clone();
                db_service::update_song(&song)?;
                Ok(())
                

            },
            EditorType::Playlist(id) =>{
                let mut playlist = db_service::get_playlist_by_id(id)?;
                playlist.list_name = self.items[0].1.clone();
                playlist.author = self.items[1].1.clone();
                playlist.genre = self.items[2].1.clone();
                db_service::update_playlist(&playlist)?;
                Ok(())
            },
            EditorType::NewPlaylist => {
                let mut playlist = db_service::Playlist{
                    list_id : 0,
                    list_name : self.items[0].1.clone(),
                    is_user_created : true,
                    author: String::from(""),
                    ..Default::default()
                };
                db_service::add_playlist(playlist)?;
                return Ok(())

            },
            EditorType::AddToPlaylist(_song_id) =>{
                Ok(())
            }

            _ => Err(anyhow::anyhow!("updating anything other than song not implemented")),
        }

    }
    pub fn back_space(&mut self){
        match self.table_state.selected_cell(){
            Some(tup) => {
                let i = tup.0;
                _ = self.items[i].1.pop();
            }
            None => (),
        };

    }
    pub fn append_to_item(&mut self, c: char){
        match self.table_state.selected_cell(){
            Some(tup) => {
                let i = tup.0;
                _ = self.items[i].1.push(c);
            },
            None => (),
        }
    }


    pub fn handle_keys(&mut self,key: KeyCode) -> Result<(),anyhow::Error>{
        match self.editor_type{
            EditorType::AddToPlaylist(song_id) =>{
                match key{
                    KeyCode::Enter=> {
                        self.save_to_playlist(song_id)?;
                    }
                    KeyCode::Char('j')=> self.table_state.select_next(),
                    KeyCode::Char('k')=> self.table_state.select_previous(),


                    _ => (),
                }

            },
            _ =>{
                match key {
                    KeyCode::Down=> self.table_state.select_next(),
                    KeyCode::Up => self.table_state.select_previous(),
                    KeyCode::Backspace => self.back_space(),
                    KeyCode::Char(c) => self.append_to_item(c),
                    _ => (),
                }
            }
        }
        Ok(())
         

    }
    pub fn save_to_playlist(&mut self,song_id: i64) -> Result<(),anyhow::Error>{
        match self.table_state.selected_cell(){
            Some(tup) => {
                let i = tup.0;
                let list_name = &self.items[i].1;
                let playlist = db_service::get_playlist_by_name(list_name)?;
                db_service::add_song_to_playlist(song_id, playlist.list_id,None)?; 
            },
            None => (),
        }
        Ok(())
    }
    
    

    
        
    
    
    

}
