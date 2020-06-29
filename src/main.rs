extern crate axgeom;
extern crate dinotree_alg;

use duckduckgeo::F32n;
use axgeom::*;

use egaku2d::*;
use duckduckgeo;

use steer::*;


use egaku2d::glutin;
use glutin::event::ElementState;
use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;




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

pub struct CommitManager{
    count:u8,
    moves:std::iter::Peekable<std::vec::IntoIter<(PlayerID,Move)>>,//The moves we are working though
    current_targets:Vec<PlayerState>,
    commits:Vec<Move> //for ourselves
}
impl CommitManager{
    fn new(playerid:PlayerID,name:[u8;8],other_players:Vec<(PlayerID,[u8;8])>,game_state:&GameState)->CommitManager{
        let mut current_targets=vec!(PlayerState{playerid,name,target:game_state.bots[playerid.0 as usize].body.pos});

        for (playerid,name) in other_players.into_iter(){
            current_targets.push(PlayerState{playerid,name, target:game_state.bots[playerid.0 as usize].body.pos });
        }
        //other_players.into_iter()
        let moves=vec!().into_iter().peekable();
        CommitManager{count:0,moves,current_targets,commits:Vec::new()}
    }

    //call this 60 times a second
    fn tick(&mut self,commit:Option<Vec2<f32>>,myplayerid:PlayerID,stream:&mut TcpStream,game_state:&GameState)->Result<&[PlayerState],ProtErr>{
        
        if self.count==0{    
            //TODO This can happen in parallel with the below if its slow
            println!("sending={:?}",&self.commits);
            ClientToServer::Commit(myplayerid,self.commits.clone()).send(stream).unwrap();
            println!("sent commits!");
            self.commits.clear();

            match ServerToClient::receive(stream)?{
                ServerToClient::Moves(a)=>{
                    let x=self.moves.next();
                    assert!(x.is_none(),"{:?}",x);
                    println!("Received moves! {:?}",a);
                    let mut p=a.into_iter().peekable();

                    //As long as there are commits that are not moves
                    while p.peek().and_then(|a|{if let CommitType::Move(_)=a.commit{None}else{Some(())}}).is_some(){
                        let Commit{playerid,commit}=p.next().unwrap();
                        match commit{
                            CommitType::Join(name)=>{
                                if playerid==myplayerid{
                                    println!("Received join message for ourselves. this is expected. ignoring");
                                }else{
                                    //dbg!("received a join from playerid={:?}",playerid);
                                    //handle
                                    let target=game_state.bots[playerid.0 as usize].body.pos;
                                    self.current_targets.push(PlayerState{playerid,name,target})
                                }
                            },
                            CommitType::Quit()=>{
                                dbg!("this player quit! {:?}",playerid);
                                let (index,_)=self.current_targets.iter().enumerate().find(|(i,e)|e.playerid==playerid).ok_or(ProtErr)?;
                                self.current_targets.remove(index);

                            },
                            CommitType::Move(_)=>{
                                unreachable!();
                            }
                        }
                    }
                    
                    self.moves = p.map(|a|(a.playerid,a.commit.assume_move().unwrap())).collect::<Vec<_>>().into_iter().peekable();
                },
                ServerToClient::GameStateRequest=>{
                    ClientToServer::GameState(game_state.clone()).send(stream).unwrap();
                },
                _=>{
                    return Err(ProtErr)
                }
            }
        }
        if let Some(target)=commit{
            self.commits.push(Move{target,tick:self.count});
        }

        let count=self.count;
        while self.moves.peek().and_then(|(_,a)|if a.tick==count{Some(())}else{None} ).is_some(){
            let (playerid,m) = self.moves.next().unwrap();
            println!("current targets={:?}",self.current_targets);
            let p=self.current_targets.iter_mut().find(|o|o.playerid==playerid).unwrap();
            p.target=m.target;
        }

        self.count+=1;
        if self.count==18{ //300ms lag
            self.count=0;
        }

        Ok(&mut self.current_targets)
    }
}




use std::net::TcpStream;
use steer::net::*;
use steer::game::PlayerState;

pub fn make_demo(args:Vec<String>,dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Result<Demo,Box<dyn std::error::Error>> {
    let window_dim:Rect<F32n>=dim;//rect(0.0,800.0,0.0,600.0*2.0).inner_try_into().unwrap();


    use std::convert::TryInto;
    let gameid:u64=args[1].parse::<u64>()?;

    let mut game=game::Game::new();
    
    let mut stream = TcpStream::connect("localhost:3333")?;
    
    
    ClientToServer::JoinRequest(gameid,[1;8]).send(&mut stream)?;
    println!("sent join request");

    let name=[0u8;8];
    let (myplayerid,other_players)=match ServerToClient::receive(&mut stream)?{
        ServerToClient::StartNewGame(playerid)=>{
            (playerid,Vec::new())
        },
        ServerToClient::ReceiveGameState(state,playerid,other_players)=>{
            game.state=state;
            (playerid,other_players)
        },
        _=>{
            panic!("error!");
        }
    };

    let mut commit_manager=CommitManager::new(myplayerid,name,other_players,&game.state);


    println!("MY PLAYER ID={:?}",myplayerid);
    

    let wall_save={
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
        

        

        let commit=if mouse_active{
            //convert window coordicate to game coordinate
            let target=cursor.inner_into();
            let half=vec2(window_dim.x.distance().into_inner(),window_dim.y.distance().into_inner())/2.0;
            let p=game.state.bots[myplayerid.0 as usize].body.pos;
            let mtarget=-half+target+p;
            //commit_manager.add_commit(mtarget);
            Some(mtarget)
        }else{
            None
        };


        {   //TODO maybe do this in a different orer???
            let player_moves=commit_manager.tick(commit,myplayerid,&mut stream,&game.state).unwrap();
            game.step(player_moves,canvas);
        }




        //convert game coordinate to window coordinate
        let p=game.state.bots[myplayerid.0 as usize].body.pos;

        let kk=-(p.inner_into::<f32>())+vec2(window_dim.x.distance().into_inner(),window_dim.y.distance().into_inner())/2.0;
        canvas.set_global_offset( kk.into());

        
        let grid_viewport=&game.nonstate.grid_viewport;
        let bots=&game.state.bots;
        let radius=game.nonstate.radius;
        let diameter=radius*2.0;
        wall_save.uniforms(canvas,grid_viewport.spacing).with_color([0.4,0.2,0.2,1.0]).draw();


        let xx=radius*1.5;
        let mut lines=canvas.lines(radius/2.0);
        for b in bots.iter(){
            let a=vec2(b.head.rot.cos(),b.head.rot.sin());
            if b.ind.top(){
                lines.add(b.body.pos.into(),(b.body.pos-a*xx).into());
            }else if b.ind.bottom(){
                lines.add(b.body.pos.into(),(b.body.pos+a*xx).into());
            }
            
            let a=a.rotate_90deg_right();
            if b.ind.left(){
                lines.add(b.body.pos.into(),(b.body.pos-a*xx).into());
            }else if b.ind.right(){
                lines.add(b.body.pos.into(),(b.body.pos+a*xx).into());
            }
            
        }
        lines.send_and_uniforms(canvas).with_color([1.0,0.0,0.0,0.5]).draw();
        



        //Draw circles
        let mut circles = canvas.circles();
        for b in bots.iter(){
            circles.add(b.body.pos.into());
        }
        circles.send_and_uniforms(canvas,diameter-2.0).with_color([1.0, 1.0, 1.0, 1.0]).draw();



        for &PlayerState{playerid,name,target} in commit_manager.current_targets.iter(){
            fn playerid_to_color(id:PlayerID)->[f32;4]{
                let f=id.0 as f32; //between 0 and like 20
                [(f*6.2)%1.0,(f*2.4)%1.0,(f*4.8)%1.0,1.0]
            }
            let c=playerid_to_color(playerid);
            let bpos=bots[playerid.0 as usize].body.pos;

            let mut lines=canvas.lines(1.5);
            lines.add(bpos.into(),target.into());
            lines.send_and_uniforms(canvas).with_color(c).draw();

            let mut circles = canvas.circles();
            circles.add(bpos.into());
            circles.send_and_uniforms(canvas,diameter-1.0).with_color(c).draw();
        }
        
        let mut lines=canvas.lines(2.0);
        for b in bots.iter(){
            let rr=radius-1.0;
            lines.add(b.body.pos.into(),(b.body.pos+vec2(b.head.rot.cos(),b.head.rot.sin()) *rr).into() );
        }
        lines.send_and_uniforms(canvas).with_color([0.0,0.0,0.5,1.0]).draw();
              
    });
    Ok(d)
}

