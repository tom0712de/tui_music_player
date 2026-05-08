use::sqlite::Connection;
use::anyhow::anyhow;

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
pub fn get_connection() -> Result<Connection, anyhow::Error>{
    let songs = sqlite::open("../music_player_DB.sqlite")?;
    return Ok(songs)

}

pub fn init() -> Result<(), anyhow::Error>{
    let songs = get_connection()?;
    let query = "Create Table IF NOT EXISTS songs(
        song_id INTEGER PRIMARY KEY AUTOINCREMENT,
        song_name TEXT NOT NULL,
        path TEXT NOT NULL
        );";

    songs.execute(query)?;

    let query = "CREATE TABLE IF NOT EXISTS playlist(
        list_id INTEGER PRIMARY KEY AUTOINCREMENT,
        list_name TEXT NOT NULL
        );";
    songs.execute(query)?; // Playlist stored as list seperated with spaces of ids

    let query ="CREATE TABLE IF NOT EXISTS song_playlist_junction (
    id INTEGER PRIMARY KEY,
    song_id INTEGER NOT NULL,
    playlist_id INTEGER NOT NULL,
    playlist_pos INTEGER,
    UNIQUE(playlist_id, playlist_pos)
);";

   songs.execute(query)?;
    Ok(())    
    }
    


pub fn add_song(song:&Song) -> Result<(), anyhow::Error>{ //needs sql injec proof
    let db = get_connection()?;

    let mut stmt = db.prepare("INSERT INTO songs(song_name, path) VALUES (:song_name,:path);")?;
    //stmt.bind((song.song_name.as_str(),song.path.as_str()));
    stmt.bind((":song_name",song.song_name.as_str()))?;
    stmt.bind((":path",song.path.as_str()))?;
    stmt.next()?;    

//    let query = format!("INSERT INTO songs(song_name, path) VALUES ('{}','{}');",song.song_name,song.path);
    Ok(())
}

pub fn add_song_to_playlist(song_id: i64, list_id:i64, playlist_pos: Option<i64>) -> Result<(), anyhow::Error>{
    
    
    let db = get_connection()?;
    let query = match playlist_pos{

        Some(e) => {
            let mut stmt = db.prepare("INSERT INTO song_playlist_junction(song_id,playlist_id,playlist_pos) VALUES (:song_id,:playlist_id,:playlist_pos);")?;
            //stmt.bind((1,song_id),(2,list_id),(3,e));
            stmt.bind((":song_id",song_id))?;

            stmt.bind((":playlist_id",list_id))?;
            stmt.bind((":playlist_pos",e))?;
            stmt.next()?;
        }, 
        //Some(e) => format!("INSERT INTO song_playlist_junction(song_id,playlist_id,playlist_pos) VALUES ('{}','{}','{}');",song_id,list_id,e),

        None => {
            let mut stmt = db.prepare("INSERT INTO song_playlist_junction(song_id,playlist_id) VALUES (:song_id,:playlist_id);")?;
            //stmt.bind((song_id.into(),list_id.into()));
            stmt.bind((":song_id",song_id))?;
            stmt.bind((":playlist_id",list_id))?;
            stmt.next()?;
        }
    };
    Ok(())
}

pub fn is_song_in_playlist(song_id: &i64, list_id:&i64) -> Result<bool,anyhow::Error>{
    let db = get_connection()?;
    let mut  stmt= db.prepare(format!("SELECT 1 FROM song_playlist_junction WHERE playlist_id='{}' AND song_id='{}'",list_id,song_id))?;
    Ok(State::Row == stmt.next()?)     
    
}

pub fn get_song_info(index:&i64) -> Result<Song, anyhow::Error> {

    let db = get_connection()?;
    let mut song = Song{
        song_id: *index,  
        song_name: String::from("default"),
        path: String::from(""),
    };
    let mut stmt = db.prepare(format!("SELECT * FROM songs WHERE song_id='{}';",&index))?;
                                                                                                                                //
    while State::Row == stmt.next().expect(""){ // a statement has mult State(all returned rows) if
                                                // they can be iterated with if Stat::done all rows
        song.song_name = stmt.read(1).expect("Failed to read song_name");
        song.path = stmt.read(2)?;
    }  
    Ok(song)

}
pub fn get_song_by_name(song_name: &str) -> Result<Song, anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("SELECT * FROM songs WHERE song_name= :song_name;")?;
    stmt.bind((":song_name",song_name))?;
                                                                                                                                //
    //if let State::Row = stmt.next().expect("Error trying to sql"){ // a statement has mult State(all returned rows) if
    if stmt.next()? != State::Row{
        anyhow::bail!("Song '{}' not found in get_song_by_name",song_name);
    };                                             
    let song_id= stmt.read(0).expect("Failed to read song_id");
    let path = stmt.read(2).expect("Failed to read song_path");
    let  song = Song{
        song_id: song_id,  
        song_name: song_name.to_string(), 
        path:path, 
    };

      
    Ok(song)


}
// helper functions
pub fn is_path_unique(song_path:&str) -> Result<bool,anyhow::Error>{

    let db = get_connection()?;
    let mut stmt = db.prepare("SELECT * FROM songs WHERE path= :song_path ;")?;
    stmt.bind((":song_path",song_path))?;
    Ok(State::Row != stmt.next()?)//returns true if not exists false if exists
}


pub fn get_song_count() -> Result<i64, anyhow::Error>{

    let db = get_connection()?;
    let mut stmt = db.prepare(format!("SELECT COUNT(*) FROM songs;")).expect("failed to query"); 
    let _ = stmt.next(); // type of read must be declard
    Ok(stmt.read(0).unwrap())

}
pub fn is_playlist_unique(playlist_name: &str) -> Result<bool,anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("SELECT * FROM playlist WHERE list_name= :list_name ;").expect("failed_is_playlist_unnique");
    stmt.bind((":list_name",playlist_name)).expect("");
    Ok(State::Row !=  stmt.next().expect(""))
}
pub fn add_playlist(playlist: Playlist) -> Result<(), anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("INSERT INTO playlist (list_name) VALUES (:list_name);")?;
    stmt.bind((":list_name",playlist.list_name.as_str()))?;
    //let query = format!("INSERT INTO playlist (list_name) VALUES ('{}');",playlist.list_name);
    //println!("query:{}",query);
    stmt.next()?;
    Ok(())
}
pub fn get_all_playlist() -> Result<Vec<Playlist>, anyhow::Error>{ 
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

pub fn get_playlist_by_name(list_name :&str) -> Result<Playlist,anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("SELECT * FROM playlist WHERE list_name= :list_name;").expect("");
    stmt.bind((":list_name",list_name)).expect("");
    match stmt.next().expect(""){
        
        State::Done => return Err(anyhow!("Error in db_service line: 163 -> playlist by name '{list_name}' not found")),

        State::Row=>{
            let playlist = Playlist{
                list_id: stmt.read(0).expect("Error"),
                list_name: stmt.read(1).expect("Error"),
            };
            Ok(playlist)
        }
    }
}
pub fn get_songs_from_playlist(p_playlist_name: &str) -> Result<Vec<Song>,anyhow::Error>{
    let db = get_connection()?;
    let playlist = get_playlist_by_name(p_playlist_name)?;
    let mut stmt = match db.prepare(format!("SELECT * FROM song_playlist_junction WHERE playlist_id = '{}';",playlist.list_id)){
        Ok(t) => t,
        Err(e) => return Err(anyhow!("Error in db_service.rs: line 194 '{e}'")),
    };
    let mut songs = Vec::new();
    while State::Row == stmt.next()?{
        let song :i64 = stmt.read(1).expect("");
        songs.push(get_song_info(&song)?);
        

    }
    
    Ok(songs)
    
}
