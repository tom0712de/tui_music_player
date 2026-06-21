use::std::fs;
use::std::path::Path;
pub use crate::db_service;

// get Files from Folder and add new music to db
// is a recursive Function wich returns all songs from a Folder
// if it finds a Folder it calls itself on that Folder
pub fn update_songs(p_movie_path: &Path) ->Result<Vec<i64> ,anyhow::Error> {
    let mut songs = Vec::new();
    let movie_path = p_movie_path;
    for entry in match fs::read_dir(movie_path){
        Ok(t) => t,
        Err(_e) => panic!("cant acces Path"),
    }{//loop über alle Entrys im Pfad
        let entry = entry?;
        let path = entry.path();


        if path.is_dir(){ //wenn der Pfad ein dir ist wird dieses auf alles songs durchsucht mit fn
                          //update songs und dann zu playlist db hinzugefügt
           // Create standard song 
            let mut playlist = db_service::Playlist{
                list_id: 0,
                list_name: String::from(""),
                ..Default::default()
            };
            // Namen Formatieren 
            let p_path = path.clone();
            let mut playlist_name = "";
            for val in path.into_os_string().into_string().expect("Can not work with Path").split("/"){
                playlist.list_name = String::from(val.clone());
            }


            //DB adden
            //let list_id = playlist.list_id.clone();
            let playlist_name = playlist.list_name.clone();
            let songs_in_playlist = update_songs(&p_path).expect("Error trying to extract_songs");
            if songs_in_playlist.len() > 0 && db_service::is_playlist_blacklisted(&playlist_name)?{
                  
                if db_service::is_playlist_unique(&playlist.list_name)?{
                    db_service::add_playlist(playlist)?;
                }

                let list_id = db_service::get_playlist_by_name(&playlist_name).expect("temp debug stop").list_id;
                for val in songs_in_playlist{
                    if !db_service::is_song_in_playlist(&val,&list_id)?{
                        db_service::add_song_to_playlist(val,list_id, None)?;
                    }
                }
            }
        }

        //entry is a file and needs to be formated that way
        else{
            if "mp3" == path.extension().unwrap(){

            
            let song_path = path.into_os_string().into_string().expect("can not convert Path");
            let mut song_name = String::from("");
            for part in song_path.split("/"){

                song_name = String::from(part);
            }


            
            let song = db_service::Song{
                song_id: 0,
                song_name: song_name.to_string(),
                path: song_path,
            };
            if db_service::is_path_unique(&song.path)?{
                db_service::add_song(&song)?;
            }
            let re_song = db_service::get_song_by_path(&song.path).expect("Found_error");
            songs.push(re_song.song_id);

            }
                
        
        }
    }
    Ok(songs)


    
}
pub fn extract_songs(){
    
}


