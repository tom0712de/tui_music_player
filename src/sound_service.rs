pub use crate::db_service::{Song};
use::anyhow::anyhow;
use::std::fs::File;
use std::sync::{Arc, Mutex, RwLock};
use::std::io::BufReader;
use rodio::{Decoder, MixerDeviceSink, source::Source};
use::std::collections::VecDeque;

// responsibel for playback
// keeps track of wich songs are currently played 
// interacts with rodio
// is kept on a different thread
pub struct Player_service{
    pub player: rodio::Player,
    sink_handle: rodio::stream::MixerDeviceSink,
    pub next_songs: VecDeque<Song>,
    pub prev_songs: VecDeque<Song>,
    pub current_song: Option<Song>,
    }
impl Player_service{


    pub fn new()-> Result<Self,anyhow::Error>{
        let sink_handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        let player = rodio::Player::connect_new(&sink_handle.mixer());
        Ok(Self{
            sink_handle : sink_handle,
            player: player,
            next_songs: VecDeque::new(),
            prev_songs: VecDeque::new(),
            current_song: None,
        }
        )
    }


    //responsible for checking if  a new song should be added 
    //called in the main loop of the thread
    pub fn update_current_song(&mut self) -> Result<Option<Song>,anyhow::Error> {     
        if !self.next_songs.is_empty(){
            if self.player.empty(){
                let i = self.next_songs.pop_front().take();
                return Ok(self.play_song(i.unwrap())?)
            }
        }
        Ok(None)
    }


    pub fn add_song_to_queue(&mut self, song:Song){
        self.next_songs.push_back(song);
    }

    // add song to rodio 
    // song gets played
    pub fn play_song(&mut self, song: Song) -> Result<Option<Song>,anyhow::Error>{
        let file = BufReader::new(File::open(song.path.clone())?);
        let source = Decoder::try_from(file)?;
        self.player.append(source);
        self.player.play();
        match self.current_song.take(){
            Some(i) => self.prev_songs.push_back(i),
            _ => (),
        }
        Ok(Some(song))
    }


    pub fn play_pause(&mut self){
        if self.player.is_paused(){
            self.player.play();
        }
        else{
            self.player.pause();
        }
    }


    pub fn add_songs_to_queue(&mut self, songs: Vec<Song>) -> Result<(), anyhow::Error>{
        for song in songs{
            self.add_song_to_queue(song);
        }
        Ok(())
    }


    pub fn skip_song(&mut self) {
        self.player.clear();
    }
    
    
}
// abstraction layer to controll the player 
// ,wich is on a different thread
pub struct PlayerController{
    pub tx: std::sync::mpsc::Sender<SoundControlls>,
    pub thread_handler: std::thread::JoinHandle<()>,
    pub next_songs: Arc<RwLock<Option<Vec<i64>>>>,
    pub current_song: Arc<RwLock<Option<i64>>>,


}
impl PlayerController{
    pub fn new() -> Self{
        let (tx, rx) = std::sync::mpsc::channel();
        
        let current_song:Arc<RwLock<Option<i64>>> = Arc::new(RwLock::new(None));
        let next_songs:Arc<RwLock<Option<Vec<i64>>>> = Arc::new(RwLock::new(None));
        let next_songs_clone = Arc::clone(&next_songs);
        let cur_song = Arc::clone(&current_song);

        //thread wich runs all the player stuff
        let thread_handler = std::thread::spawn(move || {
            let mut player = Player_service::new().unwrap();
            loop{
                //check if inputs were sent
                if let Ok(input) = rx.try_recv(){
                match input{
                    SoundControlls::PlayPause => player.play_pause(),
                    SoundControlls::AddSong(i) => player.add_song_to_queue(i),
                    SoundControlls::SkipSong => player.skip_song(),
                    SoundControlls::AddPlaylist(songs) => player.add_songs_to_queue(songs).expect(""),
                    _ => (), 
                }
                //check if a new song should be played
                }
                // update current song on ARC
                match player.update_current_song().unwrap(){
                    Some(i) => {
                        let mut  guard = cur_song.write().expect("");
                        *guard = Some(i.song_id);
                    },
                    None => (),
                }
                //---write next songs to ARC
                let mut n_songs: Vec<i64> = vec![];
                for item in &player.next_songs{
                    n_songs.push(item.song_id);
                }
                let mut should_write: bool = false;
                {
                    if let Ok(guard) = next_songs_clone.try_read(){
                        match guard.as_ref(){
                            Some(i) =>{
                                if i != &n_songs{
                                    should_write = true;
                                }
                            },
                            None => should_write = true,
                        }
                    }
                }
                if should_write {
                    let mut guard = next_songs_clone.write().expect("");
                    *guard= Some(n_songs);
                }   
                //---
                std::thread::sleep(std::time::Duration::from_millis(10));
                }
            });


        Self{
            tx: tx,
            thread_handler: thread_handler,
            current_song : current_song, 
            next_songs:next_songs,


        }
    }


    pub fn controll(&mut self, controlls: SoundControlls) -> Result<(),anyhow::Error>{
        self.tx.send(controlls)?;
        Ok(())

    }

}
pub enum SoundControlls{
    PlayPause,
    SkipSong,
    PrevSong,
    AddSong(Song),
    AddPlaylist(Vec<Song>)


}


