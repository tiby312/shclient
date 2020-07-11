extern crate axgeom;
extern crate dinotree_alg;

use duckduckgeo::F32n;
use axgeom::*;

use egaku2d::*;
use duckduckgeo;

use steer::*;
use steer::game::NetGameState;

use egaku2d::glutin;
use glutin::event::ElementState;
use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;

use steer::net::PlayerState;


pub struct Demo(Box<dyn FnMut(Vec2<F32n>,bool, &mut SimpleCanvas, bool)>);
impl Demo {
    pub fn new(func: impl FnMut(Vec2<F32n>, bool,&mut SimpleCanvas, bool) + 'static) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(&mut self, point: Vec2<F32n>, mouse_active:bool,sys: &mut SimpleCanvas, check_naive: bool) {
        self.0(point, mouse_active,sys, check_naive);
    }
}


use std::env;

fn main()-> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();


    let area = vec2(800, 600);

    let events_loop = glutin::event_loop::EventLoop::new();

    //let window_dim=[800*2,600*2];
    let mut sys = egaku2d::WindowedSystem::new(area.into(), &events_loop,"dinotree_alg demo");

    let r=rect(0.,area.x as f32,0.,area.y as f32);
    let mut game=make_demo(args,r.inner_try_into().unwrap(),&mut sys.canvas_mut())?;




    let mut mouse_active=false;
    let mut cursor = vec2same(0.);
    let mut timer = egaku2d::RefreshTimer::new(16);
    events_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Released {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => {
                                *control_flow = ControlFlow::Exit;
                            }
                            Some(VirtualKeyCode::N) => {
                               
                            }
                            Some(VirtualKeyCode::C) => {
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_logical_size) => {}
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                } => {
                    //let dpi=sys.get_hidpi_factor();
                    //let glutin::dpi::PhysicalPosition { x, y } = logical_position.to_physical(dpi);
                    cursor = vec2(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    ..
                } => {
                    if button == glutin::event::MouseButton::Left {
                        match state {
                            glutin::event::ElementState::Pressed => {
                                mouse_active=true;
                            }
                            glutin::event::ElementState::Released => {
                                //mouse_active=true;
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                if timer.is_ready() {
                    let k = sys.canvas_mut();
                    k.clear_color([0.2, 0.8, 0.2]);
                    game.step(cursor.inner_try_into().unwrap(),mouse_active, k, false);
                    sys.swap_buffers();

                    mouse_active=false;
                }
            }
            _ => {}
        }
    });
    //make_demo()
}







use steer::game::GameState;





pub struct PreMove{
    mycommits:Vec<Move>,
    game_state_req:bool,
    count:u8
}
impl PreMove{
    fn new()->Self{
        PreMove{mycommits:Vec::new(),game_state_req:false,count:0}
    }

    fn advance(&mut self,playerid:PlayerID,my_commit:Option<Vec2<f32>>,game:&GameState)->Option<ClientToServer>{
        if let Some(target)=my_commit{
            let tick=self.count;
            self.mycommits.push(Move{tick,target});
        }

        let c=if self.count==FRAME_LENGTH-1{
            use core::convert::TryFrom;
        
            
            let netstate:NetGameState=TryFrom::try_from(game.clone()).unwrap();
            let hash=netstate.make_hash();

            let state=if self.game_state_req{
                self.game_state_req=false;
                Some(netstate)
            }else{
                None
            };

            //send out
            let moves=self.mycommits.clone(); //TODO optimize
            Some(ClientToServer::Commit{playerid,moves,state,hash})
        }else{
            None
        };
        self.count+=1;
        c
    }
}



pub struct MoverIter<'a>{
    tick:u8,
    moves:&'a mut std::iter::Peekable<std::vec::IntoIter<(PlayerID,Move)>>
}
//TODO implement more traits
impl Iterator for MoverIter<'_>{
    type Item=(PlayerID,Vec2<f32>);
    fn next(&mut self)->Option<Self::Item>{
        if let Some((_,m))=self.moves.peek(){
            if m.tick!=self.tick{
                None
            }else{
                let (p,m)=self.moves.next().unwrap();
                Some((p,m.target))
            }
        }else{
            None
        }
    }
}


const FRAME_LENGTH:u8=5;


enum SessionResult{
    Finished,
    NotFinished
}
pub struct MoveSession{
    count:u8,
    moves:std::iter::Peekable<std::vec::IntoIter<(PlayerID,Move)>>,
    events:Option<Vec<PlayerEvent>>,
}

impl MoveSession{
    fn new(events:Vec<PlayerEvent>,moves:Vec<(PlayerID,Move)>,game_state_req:bool)->MoveSession{
        MoveSession{events:Some(events),count:0,moves:moves.into_iter().peekable()}
    }
    fn advance_game_state(&mut self,game:&mut game::Game,canvas:&mut SimpleCanvas)-> SessionResult {
        
        
        let m=MoverIter{ tick: self.count,moves:&mut self.moves};
        game.step(self.events.take(),m,canvas);

        self.count+=1;
        if self.count==FRAME_LENGTH{
            assert!(self.moves.next().is_none());
            SessionResult::Finished
        }else{
            SessionResult::NotFinished
        }
    }
}





struct PlayerStream(TcpStream);
impl PlayerStream{
    fn get_mut(&mut self)->&mut TcpStream{
        &mut self.0
    }
    fn recv(&mut self)->Result<ServerToClient,ProtErr>{
        ServerToClient::receive(self.get_mut())
    }
}
impl Drop for PlayerStream{
    fn drop(&mut self){
        ClientToServer::Quit.send(&mut self.0);
    }
}




pub struct Manager{
    playerid:PlayerID,
    gameid:u64,
    name:PlayerName,
    game:game::Game,
    premove:PreMove,
    sess:Option<MoveSession>
}
impl Manager{

    fn prep(gameid:u64,name:PlayerName)->ClientToServer{
        ClientToServer::JoinRequest{gameid,name}
    }
    fn new(gameid:u64,name:PlayerName,a:ServerToClient,b:ServerToClient,canvas:&mut SimpleCanvas)->Manager{
        
        let (playerid,game)=match a{
            ServerToClient::StartNewGame(playerid)=>{
                (playerid,game::Game::new())
            },
            ServerToClient::ReceiveGameState{mut metastate,commits,playerid}=>{
                //set the initial players
                //core::mem::swap(&mut player_states.current_targets,&mut metastate.existing_players);
                let mut game=game::Game::new();
                game.state=metastate.into();
                
                
                let mut m=MoveSession::new(Vec::new(),commits,false);
                //Advance the word by one more since the hash is always a hash of one tick behind (for performance)
                let k=m.advance_game_state(&mut game,canvas);
                
                
                (playerid,game)
            },
            _=>{
                panic!("error!");
            }
        };

        

        let sess=if let ServerToClient::ServerClientNominal{playerevents,commits,game_state:None}=b{

                //handle joins/quits
                Some(MoveSession::new(playerevents,commits,false))
        }else{
            panic!("errrr")
        };

        Manager{playerid,gameid,name,game,premove:PreMove::new(),sess}


    }

    fn premove(&mut self,mycommit:Option<Vec2<f32>>)->Option<ClientToServer>{
        if let Some(c)=self.premove.advance(self.playerid,mycommit,&self.game.state){
            self.premove=PreMove::new();
            Some(c)
        }else{
            None
        }
    }

    //only call this if premove() returned some.
    fn recv(&mut self,s:Option<ServerToClient>,canvas:&mut SimpleCanvas){
        

        if let SessionResult::Finished=self.sess.as_mut().unwrap().advance_game_state(&mut self.game,canvas){
            println!("game tick={:?}!!!",self.game.state.tick);

            
            use ServerToClient::*;
            match s.unwrap(){
                ServerClientNominal{playerevents,commits,game_state}=>{
                    let respond=if let Some(g)=game_state{
                        if g.source_player==self.playerid{
                            true
                        }else{
                            false
                        }
                    }else{
                        false
                    };

                    self.premove.game_state_req=respond;

                    //handle joins/quits
                    self.sess=Some(MoveSession::new(playerevents,commits,respond));
                },
                ReceiveGameState{metastate,commits,playerid}=>{
                    panic!("received game state?? {:?}",(metastate,commits,playerid));
                },
                StartNewGame(playerid)=>{
                    panic!("received new player?? {:?}",playerid);

                }
            }
                
        }else{
            assert!(s.is_none());
        }
            
    }
}

        


use std::net::TcpStream;
use steer::net::*;

pub fn make_demo(args:Vec<String>,dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Result<Demo,Box<dyn std::error::Error>> {
    let window_dim:Rect<F32n>=dim;//rect(0.0,800.0,0.0,600.0*2.0).inner_try_into().unwrap();


    let gameid:u64=args[1].parse::<u64>()?;

    //let mut game=game::Game::new();
    
    let mut stream = PlayerStream(TcpStream::connect("localhost:3333")?);
    
    
    let myname=PlayerName([0u8;8]);    
    
    Manager::prep(gameid,myname).send(stream.get_mut())?;

    //println!("sent join request");
    let a1=stream.recv()?;
    let a2=stream.recv()?;
    let mut m=Manager::new(gameid,myname,a1,a2,canvas);
    

    
    
    
    let wall_save={
        let game=&m.game;    

        let walls=&game.nonstate.walls;
        let grid_viewport=&game.nonstate.grid_viewport;
        let mut squares=canvas.squares();
         for x in 0..walls.dim().x {
            for y in 0..walls.dim().y {
                let curr=vec2(x,y);
                if walls.get(curr) {
                    let pos=grid_viewport.to_world_center(vec2(x, y));
                    squares.add(pos.into());
                }
            }
        }
        squares.save(canvas)
    };

    

    let d=Demo::new(move |cursor, mouse_active,canvas, _check_naive| {
        
        let mycommit=if mouse_active{
            let myplayerid=m.playerid;
            let target=cursor.inner_into();
            let half=vec2(window_dim.x.distance().into_inner(),window_dim.y.distance().into_inner())/2.0;
            let p=m.game.state.bots[myplayerid.0 as usize].0.body.pos;
            let mtarget=-half+target+p;
            Some(mtarget)
        }else{
            None
        };
        

        if let Some(s)=m.premove(mycommit){
            s.send(stream.get_mut());
            let a=ServerToClient::receive(stream.get_mut()).unwrap();
            m.recv(Some(a),canvas);
        }else{
            m.recv(None,canvas);
        }


        let myplayerid=m.playerid;
        let game=&m.game;            

        //convert game coordinate to window coordinate
        let p=game.state.bots[myplayerid.0 as usize].0.body.pos;

        let kk=-(p.inner_into::<f32>())+vec2(window_dim.x.distance().into_inner(),window_dim.y.distance().into_inner())/2.0;
        canvas.set_global_offset( kk.into());

        
        let grid_viewport=&game.nonstate.grid_viewport;
        let bots=&game.state.bots;
        let radius=game.nonstate.radius;
        let diameter=radius*2.0;
        wall_save.uniforms(canvas,grid_viewport.spacing).with_color([0.4,0.2,0.2,1.0]).draw();

        
        let xx=radius*1.5;
        let mut lines=canvas.lines(radius/2.0);
        for (b,t) in bots.iter(){
            let a=vec2(b.head.rot.cos(),b.head.rot.sin());
            
            if t.thrust_ind.top(){
                lines.add(b.body.pos.into(),(b.body.pos-a*xx).into());
            }else if t.thrust_ind.bottom(){
                lines.add(b.body.pos.into(),(b.body.pos+a*xx).into());
            }
            
            let a=a.rotate_90deg_right();
            if t.thrust_ind.left(){
                lines.add(b.body.pos.into(),(b.body.pos-a*xx).into());
            }else if t.thrust_ind.right(){
                lines.add(b.body.pos.into(),(b.body.pos+a*xx).into());
            }
            
        }
        lines.send_and_uniforms(canvas).with_color([1.0,0.0,0.0,0.5]).draw();
        



        //Draw circles
        let mut circles = canvas.circles();
        for (b,_) in bots.iter(){
            circles.add(b.body.pos.into());
        }
        circles.send_and_uniforms(canvas,diameter-2.0).with_color([1.0, 1.0, 1.0, 1.0]).draw();



        for PlayerState{playerid,name,target} in game.state.player_states.iter(){
            let _name=name;
            let target=*target;
            fn playerid_to_color(id:PlayerID)->[f32;4]{
                let f=id.0 as f32; //between 0 and like 20
                [(f*6.2)%1.0,(f*2.4)%1.0,(f*4.8)%1.0,1.0]
            }
            let c=playerid_to_color(*playerid);
            let bpos=bots[playerid.0 as usize].0.body.pos;

            let mut lines=canvas.lines(1.5);
            lines.add(bpos.into(),target.into());
            lines.send_and_uniforms(canvas).with_color(c).draw();

            let mut circles = canvas.circles();
            circles.add(bpos.into());
            circles.send_and_uniforms(canvas,diameter-1.0).with_color(c).draw();
        }
        
        let mut lines=canvas.lines(2.0);
        for (b,_) in bots.iter(){
            let rr=radius-1.0;
            lines.add(b.body.pos.into(),(b.body.pos+vec2(b.head.rot.cos(),b.head.rot.sin()) *rr).into() );
        }
        lines.send_and_uniforms(canvas).with_color([0.0,0.0,0.5,1.0]).draw();
              
    });
    Ok(d)
}

