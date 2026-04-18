use::sqlite::Connection;
use::std::io::Error;
use::std::io::ErrorKind;

use::sqlite;
use::sqlite::State;

#[derive(Debug)]
pub struct Song {
    pub song_id: i64,
    pub song_name: String,
    pub path: String,

}
pub struct Playlist{
    pub list_id: i64,
    pub list_name: String,
}
//Function is used in this module only 
pub fn get_connection() -> Result<Connection, Error>{
    let songs = match sqlite::open("../music_player_DB.sqlite"){
        Ok(t) => t,
        Err(_e) => return Err(std::io::Error::new(ErrorKind::Other, "failed to establish Connection to Database")) , 
    };
    return Ok(songs);

}

pub fn init() -> Result<(), Error>{
    let songs = get_connection()?;
    let query = "Create Table IF NOT EXISTS songs(
        song_id INTEGER PRIMARY KEY AUTOINCREMENT,
        song_name TEXT NOT NULL,
        path TEXT NOT NULL
        );";

    let _ = songs.execute(query);

    let query = "CREATE TABLE IF NOT EXISTS playlist(
        list_id INTEGER PRIMARY KEY AUTOINCREMENT,
        list_name TEXT NOT NULL,
        );";
    let _ = songs.execute(query); // Playlist stored as list seperated with spaces of ids
    

    let query = "CREATE TABLE IF NOT EXISTS song_playlist_junction(
        id INTEGER PRIMARY Key,
        song_id INTEGER NOT NULL,
        playlist_id INTEGER NOT NULL),
        playlist_pos INTEGER );"; // potential error here A playlist could have different songs on same pos
    let _ = songs.execute(query);
    
                                  //
                                  //
    Ok(())    
    }
    


pub fn add_song(song:&Song) -> Result<(), Error>{
    let db = get_connection()?;
    let query = format!("INSERT INTO songs(song_name, path) VALUES ('{}','{}');",song.song_name,song.path);
    let _ = match db.execute(&query){
        Ok(t) => t,
        Err(_e) => panic!("failed to add song"),
    };
    Ok(())
}

pub fn add_song_to_playlist(song_id: i64, list_id:i64, playlist_pos: Option<i64>) -> Result<(), Error>{
    

    let db = get_connection()?;
    let query = match playlist_pos{
        Some(e) => format!("INSERT INTO song_playlist_junction(song_id,playlist_id,playlist_pos) VALUES ('{}','{}','{}');",song_id,list_id,e),
        None => format!("INSERT INTO song_playlist_junction(song_id,playlist_id) VALUES ('{}','{}');",song_id,list_id),

    };
    match db.execute(&query){
        Ok(_t) => (),
        Err(e) => return Err(Error::new(ErrorKind::Other,"Error in db_service: Line 81 '{e}'")),
    };
    Ok(())
}


pub fn get_song_info(index:&i64) -> Result<Song, std::io::Error> {

    let db = get_connection()?;
    let mut song = Song{
        song_id: *index,  
        song_name: String::from("default"),
        path: String::from(""),
    };
    let mut stmt = match db.prepare(format!("SELECT * FROM songs WHERE song_id='{}';",&index)){
        Ok(t) => t,
        Err(e) => return Err(Error::new(ErrorKind::Other,"Error in db_service: Line 97 '{e}'")),
    }; //-> Returns <Statement>
                                                                                                                                //
    while State::Row == stmt.next().expect(""){ // a statement has mult State(all returned rows) if
                                                // they can be iterated with if Stat::done all rows
        song.song_name = stmt.read(1).expect("Failed to read song_name"); //a specific col of row can be read and song.path  = stmt.read(2).expect("Failed to read Path") ;             //returns <T> T:value indicated by db
    }  
    Ok(song)

}
pub fn get_song_by_name(song_name: &str) -> Result<Song, Error>{
   let db = get_connection()?;
   let mut song = Song{
        song_id: 0,  
        song_name: song_name.to_string(), 
        path: String::from(""),
    };
    let mut stmt = db.prepare(format!("SELECT * FROM songs WHERE song_name='{}';",song_name)).expect("failed to process SQL Query"); //-> Returns <Statement>
                                                                                                                                //
    if let State::Row = stmt.next().expect("Error trying to sql"){ // a statement has mult State(all returned rows) if
                                                // they can be iterated with if Stat::done all rows
        song.song_id= stmt.read(0).expect("Failed to read song_id");
    }  
    Ok(song)


}
// helper functions
pub fn is_path_unique(song_path:&str) -> Result<bool,Error>{

    let db = get_connection()?;
    let mut stmt = db.prepare(format!("SELECT * FROM songs WHERE path='{}';",song_path)).expect("failed to query");
    Ok(State::Row != stmt.next().expect(""))//returns true if not exists false if exists
}


pub fn get_song_count() -> Result<i64, Error>{

    let db = get_connection()?;
    let mut stmt = db.prepare(format!("SELECT COUNT(*) FROM songs;")).expect("failed to query"); 
    let _ = stmt.next(); // type of read must be declard
    Ok(stmt.read(0).unwrap())

}
pub fn is_playlist_unique(playlist_name: &str) -> Result<bool,Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare(format!("SELECT * FROM playlist WHERE list_name='{}';",playlist_name)).expect("failed to query");
    Ok(State::Row !=  stmt.next().expect(""))
}
pub fn add_playlist(playlist: Playlist) -> Result<(), Error>{
    let db = get_connection()?;
    let query = format!("INSERT INTO playlist (list_name) VALUES ('{}');",playlist.list_name);
    println!("query:{}",query);
    let _ = match db.execute(&query){
        Ok(t) => t,
        Err(e) => return Err(Error::new(ErrorKind::Other,"Error in db_service: line 152 '{e}' ")),
    };
    Ok(())
}
pub fn get_all_playlist() -> Result<Vec<Playlist>, Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare(format!("SELECT * FROM playlist")).expect("failed to query");
    let mut v = Vec::new();
    while State::Row == stmt.next().expect("here"){
        let playlist = Playlist{
            list_id :stmt.read(0).expect("Error"),
            list_name : stmt.read(1).expect("error"),
        };
        v.push(playlist);
    }
    Ok(v)
}

pub fn get_playlist_by_name(list_name :&str) -> Result<Playlist,Error>{
    let db = get_connection()?;
    let mut stmt = match db.prepare(format!("SELECT * FROM playlist WHERE list_name='{}';", list_name)){
        Ok(t) => t,
        Err(e) => return Err(Error::new(ErrorKind::Other,"Error in db_service: line 174 '{e}'")),
    };
    match stmt.next().unwrap(){
        
        State::Done => return Err(Error::new(ErrorKind::NotFound,format!("Playlist by name '{}' not found",list_name))),

        State::Row=>{
            let playlist = Playlist{
                list_id: stmt.read(0).expect("Error"),
                list_name: stmt.read(1).expect("Error"),
            };
            Ok(playlist)
        }
    }
}
pub fn get_songs_from_playlist(p_playlist_name: &str) -> Result<Vec<Song>,Error>{
    let db = get_connection()?;
    let playlist = get_playlist_by_name(p_playlist_name)?;
    let mut stmt = match db.prepare(format!("SELCET * FROM song_playlist_junction WHERE playlist_id = '{}';",playlist.list_id)){
        Ok(t) => t,
        Err(e) => return Err(Error::new(ErrorKind::Other,"Error in db_service.rs: line 194 '{e}'")),
    };
    let mut songs = Vec::new();
    while State::Row == stmt.next().expect("possible Error"){
        let song = Song{
            song_id: stmt.read(0).expect("Error"),
            song_name: stmt.read(0).expect("Error"),
            path: stmt.read(0).expect("Error",)
        };
        songs.push(song);
    }
    Ok(songs)
    
}
