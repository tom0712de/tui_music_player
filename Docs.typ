#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import fletcher.shapes: pill, rect 
= Services

#diagram(
edge-stroke: 1pt,
spacing:(7em,7em),
node-fill: white,
node-stroke:black,
node((0,0),[MusikDateien],name: <Files>, shape: rect),
node((0,3),[DB_API], name: <DB>),
node((1,1),[Main],name:<Main>),
node((3,1),[TUI],name:<TUI>),
node((1,3),[MusicPlayer],name:<MusicPlayer>),
node((2,4),[SongsDB ],name:<Songs>),
node((1,4),[Playlist/Alben],name:<PlayList>),

edge(<PlayList>,<DB>,"->",bend: 20deg,[getPlaylist]),
edge(<Songs>,<DB>,"->",[getMeta]),
edge(<MusicPlayer>,<Main>,[Was jetzt Spielt],"->",bend: -20deg),
edge(<Main>,<MusicPlayer>,[User Input +Path],"->",bend: -20deg),
edge(<TUI>,<Main>,"->",`Clicked / User Input`,bend: -20deg,),
edge(<Main>,<TUI>,"->",`MetaDaten Files etc`,bend: -20deg,),
edge(<DB>,<Main>,[Metadaten],"->",bend: 20deg),
edge(<Files>,<DB>,"->",bend: 20deg,[FilePath ]),
edge(<Files>,<MusicPlayer>,"->",[Play],bend: 20deg)


)


==


