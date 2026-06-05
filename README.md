# Rusty Music Player
A simple offline tui Music player written in Rust.
## features 
- automaticly searches through a given dir to create a .sqlite DB with all Albums/Songs
- allows editing of song Names and editing of the Songs order inside the playlist
## Goals
- make install easier / no hard coded paths
- add a way to create playlists inside the editor
- cava integration
## Install guide
- clone the repo
- build with cargo build --release
- run the programm one time (this will create a default config)
- add the location of the music and of DB in the config
- Config is located under .config/rusty-music/default-config.toml
- run the programm again

