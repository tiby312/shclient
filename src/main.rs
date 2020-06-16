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


mod maps{
    use axgeom::vec2;
    use duckduckgeo::grid::*;
    pub const GRID_STR1:Map<'static>= Map{dim:vec2(16,12),str:"\
████████████████
█  █           █
█  █   █       █
█  █  █  █     █
█  █  █        █
█   █  █       █
█     █        █
█ █  █   █     █
█   █   █      █
█        █     █
█         █    █
████████████████
"};
}





pub struct Demo(Box<dyn FnMut(Vec2<F32n>,bool, &mut SimpleCanvas, bool)>);
impl Demo {
    pub fn new(func: impl FnMut(Vec2<F32n>, bool,&mut SimpleCanvas, bool) + 'static) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(&mut self, point: Vec2<F32n>, mouse_active:bool,sys: &mut SimpleCanvas, check_naive: bool) {
        self.0(point, mouse_active,sys, check_naive);
    }
}




fn main(){
    let area = vec2(800*2, 600);

    let events_loop = glutin::event_loop::EventLoop::new();

    //let window_dim=[800*2,600*2];
    let mut sys = egaku2d::WindowedSystem::new(area.into(), &events_loop,"dinotree_alg demo");

    let r=rect(0.,area.x as f32,0.,area.y as f32);
    let mut game=make_demo(r.inner_try_into().unwrap(),&mut sys.canvas_mut());




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
                                mouse_active=false;
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
                }
            }
            _ => {}
        }
    });
    //make_demo()
}





#[derive(Eq,PartialEq,Copy,Clone)]
struct PlayerId(u32);


struct GameState{
    bots:Vec<Bot>,
    players:Vec<(PlayerId,usize,Vec2<f32>)> //playerid, and index into the bot that the player controls
}

struct Game{
    state:GameState
}

impl Game{
    fn new()->Game{
        unimplemented!()
    }
    fn step(&mut self,moves:&[(PlayerId,Option<Vec2<f32>>)]){

        //TODO
        /////////////////
        ///////////////
        ////////////
        ///////////
        //
        //
        // MAKE THE PLAYER ID THE SAME THING AS THE BOT INDEX!!!!
        //
        let state=&mut self.state;

        for (mid,t) in moves.iter(){
            if let Some((id,_,target))=state.players.iter_mut().find(|(id,_,_)|id==mid){
                if let Some(new_target)=t{
                    *target=*new_target;
                }else{
                    //do nothing
                }
            }else{
                use std::collections::HashSet;
                let index={
                    let mut indicies:HashSet<_>=state.players.iter().map(|(_,a,_)|*a).collect();
                    let h:HashSet<_>=(0..state.bots.len()).collect();

                    let difference=h.difference(&indicies);
                    let mut vv:Vec<_>=difference.map(|a|*a).collect();
                    vv.sort();
                    vv[0]
                };

                if let Some(new_t)=t{
                    state.players.push((*mid,index,*new_t));
                    //new player with target.
                }else{
                    let nn=state.bots[index].pos;
                    state.players.push((*mid,index,nn));
                    //new player with no target
                }
                
            }

        }
    }
}







pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let window_dim:Rect<F32n>=dim;//rect(0.0,800.0,0.0,600.0*2.0).inner_try_into().unwrap();


    let dim:Rect<F32n>=rect(0.0,800.0*2.0,0.0,600.0*2.0).inner_try_into().unwrap();

    let radius = 8.0;
    let diameter=radius*2.0;

    let mut counter: f32=0.0;



    let mut solver=seq_impulse::CollisionVelocitySolver::new();

    //let grid_dim=vec2(70,30*2);
    //let grid=GridViewPort{origin:vec2(0.0,0.0),spacing:dim.x.inner_into::<f32>().distance()/grid_dim.x as f32};
    
    let walls = duckduckgeo::grid::Grid2D::from_str(maps::GRID_STR1);
    let grid_viewport=duckduckgeo::grid::GridViewPort{origin:vec2(0.0,0.0),spacing:dim.x.distance().into_inner()/maps::GRID_STR1.dim.x as f32};

    let mut bots:Vec<_>=(0..3000).map(|_|walls.pick_empty_spot().unwrap()).map(|pos|Sheep::new(grid_viewport.to_world_center(pos))).collect();


    let wall_save={
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


    let mut mtarget=vec2(0.0,0.0);
    //let mut cursor_buffer=CursorBuffer::new();
    Demo::new(move |cursor, mouse_active,canvas, _check_naive| {
        
        let mut tree=dinotree_alg::collectable::CollectableDinoTree::new(&mut bots,|b| {
            Rect::from_point(b.body.pos, vec2same(radius))
                .inner_try_into::<ordered_float::NotNan<f32>>()
                .unwrap()
        });

        
    



        for b in tree.get_bots_mut().iter_mut() {
            b.reset();
        }

        let s=SheepProperty{lateral_mag:0.1,rotational_mag:0.05,lateral_fric:0.02};
            
        
        //handle dog
        {
            //sheep and dog move at same speed
            //let mag=0.05;

            //move dog to cursor.
            let dog=&mut tree.get_bots_mut()[0];
        
            if mouse_active{
                let target=cursor.inner_into();
                let half=vec2(window_dim.x.distance().into_inner(),window_dim.y.distance().into_inner())/2.0;
            
                mtarget=-half+target+dog.body.pos;

            }

            

            let (r,l)=steer_and_rotate_to_point(dog,canvas,&s,&mtarget);
            dog.apply_steering(canvas,&r,&l,&s);

            

            
            
            //make all sheep avoid dog.
            let dog_avoid_radius=150.0;
            
            let vv = vec2same(dog_avoid_radius).inner_try_into().unwrap();
            let dog_center=dog.body.pos;
            let rect=axgeom::Rect::from_point(dog_center, vv).inner_try_into().unwrap();
            let dog_ptr=dog as *const Sheep;
            tree.get_mut().for_all_in_rect_mut(&rect,| b| {
                
                if *b as *const Sheep!=dog_ptr{
                    let dd=(b.body.pos-dog_center).magnitude();
                    if dd<dog_avoid_radius{
                        let (r,l)=b.avoid(&dog_center,&s,dog_avoid_radius);
                        b.apply_steering(canvas,&r,&l,&s);
                
                        /*
                        let k=b.body.seek(&dog_center,mag);
                        let dd=(dog_avoid_radius-dd)/dog_avoid_radius;
                        assert!(dd>=0.0 && dd<=1.0,"{:?}",dd);
                        b.body.vel-=k*dd;
                        */
                    }
                    assert!(!b.body.vel.is_nan());
                }
                        
            });
            
        }


        //integrate forces
        use rayon::prelude::*;
        tree.get_bots_mut().par_iter_mut().for_each(|b|{            
            
            let new_vel=b.body.vel+b.body.vel.normalize_to(1.0)*-s.lateral_fric;
            if !new_vel.is_nan(){
                b.body.vel=new_vel;
            }
            
        });


        for b in tree.get_bots_mut().iter_mut(){
            duckduckgeo::collide_with_border(&mut b.body.pos,&mut b.body.vel,dim.as_ref(),0.5);
        }

        solver.solve(radius,&grid_viewport,&walls,&mut tree,|a|&a.body.pos,|a|&mut a.body.vel);
        
        
        //integrate positions
        for b in tree.get_bots_mut().iter_mut() {
            b.head.rot+=b.head.rot_vel;

            //assert!(!b.body.vel.is_nan());
            b.body.pos+=b.body.vel;
        }
        
        counter+=0.001;
     
        /*
        //Draw grass
        let mut rects = canvas.rects();
        for x in 0..grass.dim().x{
            for y in 0..grass.dim().y{
                //if grass.contains(vec2(x,y)){
                    let m=grass.get(vec2(x,y));
                    let tl=grid.to_world_center(vec2(x,y));
                    let dd=grid.cell_radius()*0.5*(*m as f32)/(u8::MAX as f32);
                    rects.add(axgeom::Rect::from_point(tl,vec2same(dd)).into());
                //}
            }
        }
        rects.send_and_uniforms(canvas).with_color([0.6,0.4,0.1,0.2]).draw();
        */

        let kk=-(bots[0].body.pos.inner_into::<f32>())+vec2(window_dim.x.distance().into_inner(),window_dim.y.distance().into_inner())/2.0;
        canvas.set_global_offset( kk.into());

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



        let mut circles = canvas.circles();
        circles.add(bots[0].body.pos.into());
        circles.send_and_uniforms(canvas,diameter-1.0).with_color([1.0,0.0,0.0,1.0]).draw();

        
        let mut lines=canvas.lines(2.0);
        for b in bots.iter(){
            let rr=radius-1.0;
            lines.add(b.body.pos.into(),(b.body.pos+vec2(b.head.rot.cos(),b.head.rot.sin()) *rr).into() );
        }
        lines.send_and_uniforms(canvas).with_color([0.0,0.0,0.5,1.0]).draw();
              
    })
}

