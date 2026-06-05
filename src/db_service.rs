use::sqlite::Connection;
use::anyhow::anyhow;

use::sqlite;
use::sqlite::State;
pub use crate::Config;
#[derive(Debug)]

pub struct Song {
    pub song_id: i64,
    pub song_name: String,
    pub path: String,

}

#[derive(Default)]
pub struct Playlist{
    pub list_id: i64,
    pub list_name: String,
    pub is_user_created: bool, 
}
//Function is used in this module only 
pub fn get_connection() -> Result<Connection, anyhow::Error>{

    let cfg: Config::Config = confy::load("Rusty-Music",None).expect("failed to load config");
    let songs = sqlite::open(cfg.db_path.as_str())?;

    //let songs = sqlite::open("/mnt/HDD/music/music_player_DB.sqlite")?;
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
        list_name TEXT NOT NULL,
        is_user_created INTEGER 
        );";
    songs.execute(query)?; // Playlist stored as list seperated with spaces of ids

    let query ="CREATE TABLE IF NOT EXISTS song_playlist_junction (
    id INTEGER PRIMARY KEY,
    song_id INTEGER NOT NULL,
    playlist_id INTEGER NOT NULL,
    playlist_pos INTEGER);";

   songs.execute(query).expect("can not create junction");
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
pub fn update_song(song: &Song) -> Result<(), anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        UPDATE songs
        SET Song_name = :song_name

        WHERE song_id = :song_id ;
        ")?;
    stmt.bind((":song_name",song.song_name.as_str()))?;
    stmt.bind((":song_id",song.song_id))?;
    stmt.next()?;
    Ok(())

}
pub fn update_playlist(playlist: &Playlist) -> Result<(),anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        UPDATE playlist
        SET list_name = :list_name
        WHERE list_id = :list_id
        ")?;
    stmt.bind((":list_name",playlist.list_name.as_str()))?;
    stmt.bind((":list_id",playlist.list_id))?;
    stmt.next()?;
    Ok(())
}
pub fn mov_song_up(song: &Song,playlist_id: i64,increment: i64) -> Result<(),anyhow::Error>{
    let pos = get_song_pos(song.song_id)?;
    let db = get_connection()?;
    let new_pos = pos + increment;
    if is_song_at_pos(new_pos,playlist_id)? && new_pos > 0{
        let mut mov_down = db.prepare("
            UPDATE song_playlist_junction
            SET playlist_pos = :pos
            WHERE playlist_id = :playlist_id AND playlist_pos =:pos_new;
            ")?;
        let mut mov_up= db.prepare("
            Update song_playlist_junction
            Set playlist_pos = :new_pos
            Where song_id = :song_id;
        ").expect("Error in sql statement");

        mov_up.bind((":new_pos",new_pos)).expect("check1");
        mov_up.bind((":song_id",song.song_id))?;


        mov_down.bind((":playlist_id",playlist_id))?;
        mov_down.bind((":pos_new",new_pos))?;
        mov_down.bind((":pos",pos)).expect("check2");

        mov_down.next()?;
        mov_up.next()?;



    }

    Ok(())
}
pub fn is_pos_unique(pos: i64, playlist: i64) -> Result<bool, anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        SELECT COUNT(playlist_pos)
        FROM song_playlist_junction
        WHERE playlist_pos = :pos AND playlist_id =:playlist;
        ")?;
    stmt.bind((":pos",pos))?;
    stmt.bind((":playlist",playlist))?;

    if stmt.next()? == State::Row{
        let count :i64 = stmt.read(0)?;
        return Ok(count == 1)
    }else{
        anyhow::bail!("can not figure if pos is unique");
    }
    
}
pub fn is_song_at_pos(pos:i64,playlist_id: i64) -> Result<bool,anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        SELECT COUNT(playlist_pos)
        FROM song_playlist_junction
        WHERE playlist_pos = :pos AND playlist_id =:playlist;
        ")?;
    stmt.bind((":pos",pos))?;
    stmt.bind((":playlist",playlist_id))?;

    if stmt.next()? == State::Row{
        let count :i64 = stmt.read(0)?;
        return Ok(count >0)
    }else{
        anyhow::bail!("can not figure if pos is unique");
    }

}
pub fn add_song_to_playlist(song_id: i64, list_id:i64, playlist_pos: Option<i64>) -> Result<(), anyhow::Error>{
    
    
    let db = get_connection()?;
    let query = match playlist_pos{

        Some(e) => {
            let mut stmt = db.prepare("
                INSERT INTO song_playlist_junction(song_id,playlist_id,playlist_pos)
                VALUES (:song_id,:playlist_id,:playlist_pos);
                ")?;
            stmt.bind((":song_id",song_id))?;

            stmt.bind((":playlist_id",list_id))?;
            stmt.bind((":playlist_pos",e))?;
            stmt.next()?;
        }, 
        //Some(e) => format!("INSERT INTO song_playlist_junction(song_id,playlist_id,playlist_pos) VALUES ('{}','{}','{}');",song_id,list_id,e),

        None => {
            let mut max_stmt = db.prepare("
                SELECT MAX(playlist_pos)
                FROM song_playlist_junction
                WHERE playlist_id = :playlist_id;
                ").expect("check1");
            max_stmt.bind((":playlist_id",list_id))?;
            let mut new_pos = 0;
            if max_stmt.next()? == State::Row{
                new_pos = max_stmt.read(0).expect("check3") ;
                new_pos = new_pos +1 ;
            }

            let mut stmt = db.prepare("INSERT INTO song_playlist_junction
            (song_id,playlist_id,playlist_pos)
            VALUES (:song_id,:playlist_id, :playlist_pos);").expect("Error trying to insert song into playlist");
            //stmt.bind((song_id.into(),list_id.into()));
            stmt.bind((":playlist_pos",new_pos));
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
    let mut stmt = db.prepare(format!("SELECT * FROM songs 
            WHERE song_id='{}';",&index))?;
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
pub fn get_song_by_path(song_path: &str) -> Result<Song, anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
            SELECT * FROM songs
            WHERE path = :song_path;
        ")?;
    stmt.bind((":song_path",song_path))?;
    if stmt.next()? != State::Row{
        anyhow::bail!("Song '{}' not found in get_song_by_path",song_path);
    };                                             

    let song_id= stmt.read(0).expect("Failed to read song_id");
    let song_name = stmt.read(1).expect("Failed to read song_name");
    let  song = Song{
        song_id: song_id,  
        song_name: song_name,
        path:String::from(song_path), 
    };
    Ok(song)
 


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
    let mut stmt = db.prepare("INSERT INTO playlist (list_name,is_user_created) 
        VALUES (:list_name, :is_user_created);")?;
        
    stmt.bind((":list_name",playlist.list_name.as_str()))?;
    if playlist.is_user_created{
        stmt.bind((":is_user_created","1"))?;
    }else{
        stmt.bind((":is_user_created","0"))?;
    }

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

            ..Default::default()
        };
        v.push(playlist);
    }
    Ok(v)
}

pub fn get_playlist_by_name(list_name :&str) -> Result<Playlist,anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("SELECT * 
        FROM playlist 
        WHERE list_name= :list_name ;")?;
    stmt.bind((":list_name",list_name))?;
    match stmt.next()?{
        
        State::Done => return Err(anyhow!("Error in db_service line: 163 -> playlist by name '{list_name}' not found")),

        State::Row=>{
            let byte: i64 = stmt.read(2).expect("temp debuf stop 2");
            let playlist = Playlist{
                list_id: stmt.read(0).expect("Error"),
                list_name: stmt.read(1).expect("Error"),
                is_user_created: byte == 0,
                ..Default::default() 
            };
            Ok(playlist)
        }
    }
}

pub fn get_playlist_by_id(id: i64) -> Result<Playlist,anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        SELECT * 
        FROM playlist
        WHERE list_id = :list_id;
        ").expect("Check1");
    stmt.bind((":list_id",id))?;
    if stmt.next().expect("Check2") != State::Row{
        anyhow::bail!("cant get playlist from id {}",id);
    }

    let byte: i64 = stmt.read(2)?;
    return Ok(Playlist{
    list_id : stmt.read(0)?,
    list_name : stmt.read(1)?,
    is_user_created: byte == 0,
    ..Default::default()
    })


}

pub fn get_user_created_playlist() -> Result<Vec<Playlist>,anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        SELECT * 
        FROM playlist 
        WHERE is_user_created = 1;
        ")?;
    let mut playlists: Vec<Playlist> = vec![];
    while stmt.next()? == State::Row{
        playlists.push(Playlist{
            list_id : stmt.read(0)?,
            list_name : stmt.read(1)?,
            is_user_created: true,
            ..Default::default()
        });
    }
    return Ok(playlists)
}
pub fn remove_playlist(name: &str) -> Result<(),anyhow::Error>{
    let db = get_connection()?;
    let mut stmt = db.prepare("
        DELETE FROM playlist
        WHERE list_name = :list_name ; "
    )?;
    stmt.bind((":list_name",name))?;
    stmt.next()?;

    Ok(())

}
pub fn get_song_pos(song: i64) -> Result<i64,anyhow::Error>{// returns song pos requires song_id
                                                           // returns -1 if song pos is null
    let db = get_connection()?;
    let mut stmt = db.prepare("
        SELECT playlist_pos
        FROM song_playlist_junction
        Where song_id = :song_id

        ")?;
    stmt.bind((":song_id",song))?;
    if stmt.next()? != State::Row{
        anyhow::bail!("song_id'{}' not found in song_playlist_junction",song);
    };  

    let number: i64 = stmt.read(0).expect("");
    Ok(number)


}
pub fn get_songs_from_playlist(p_playlist_name: &str) -> Result<Vec<Song>,anyhow::Error>{
    let db = get_connection()?;
    let playlist = get_playlist_by_name(p_playlist_name)?;
    let mut stmt = match db.prepare(format!("
            SELECT * FROM song_playlist_junction
            WHERE playlist_id = '{}'
            ORDER BY playlist_pos;
            ",playlist.list_id)){
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

pub fn remove_song_from_playlist(playlist_id: i64, song_id: i64) -> Result<(),anyhow::Error>{
    let db = get_connection()?;
    let song = get_song_info(&song_id)?;
    let playlist = get_playlist_by_id(playlist_id)?;
    eprintln!("playlistid: {}",playlist.list_id);

    eprintln!("song_id: {}",song.song_id);
    let mut stmt = db.prepare("
            DELETE  FROM song_playlist_junction
            WHERE playlist_id = :playlist_id AND song_id = :song_id
        ")?;
    stmt.bind((":song_id",song.song_id))?;
    stmt.bind((":playlist_id",playlist.list_id))?;
    stmt.next()?;
    Ok(())
}
