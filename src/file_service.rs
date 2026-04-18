use::std::fs;
use::std::path::Path;
pub use crate::db_service;

// get Files from Folder and add new music to db
pub fn update_songs(p_movie_path: &Path) ->Result<(Vec<i64>),anyhow::Error> {
    let mut songs = Vec::new();
    let movie_path = p_movie_path;
    for entry in match fs::read_dir(movie_path){
        Ok(t) => t,
        Err(_e) => panic!("cant acces Path"),
    }{
        let entry = entry?;
        let path = entry.path();


        if path.is_dir(){ //wenn der Pfad ein dir ist wird dieses auf alles songs durchsucht mit fn
                          //update songs und dann zu playlist db hinzugefügt
            
            let mut playlist = db_service::Playlist{
                list_id: 0,
                list_name: String::from(""),
            };
            //playlist.songs = update_songs(&path)?;
            // Namen Formatieren 
            let p_path = path.clone();
            for val in path.into_os_string().into_string().expect("Can not work with Path").split("/"){
                playlist.list_name = String::from(val);
            }

            //DB adden
            let list_id = playlist.list_id.clone();
            if db_service::is_playlist_unique(&playlist.list_name)?{
                db_service::add_playlist(playlist)?;
            }

            for val in update_songs(&p_path)?{
               db_service::add_song_to_playlist(val,list_id, None)? 
            }
        }

        //entry is a file and needs to be formated that way
        else{
            let song_path = path.into_os_string().into_string().expect("can not convert Path");
            let mut song_name = String::from("");
            for part in song_path.split("/"){
                song_name = String::from(part);
            }
            let song = db_service::Song{
                song_id: 0,
                song_name: song_name,
                path: song_path,
            };
        
            if db_service::is_path_unique(&song.path)?{
                db_service::add_song(&song)?;
            }
            let re_song = db_service::get_song_by_name(&song.song_name)?;
            //println!("here is re_song'{}'",&re_song.song_id);
            songs.push(re_song.song_id);
            
        
        }
    }
    Ok(songs)

    
}
pub fn extract_songs(){
    
}


