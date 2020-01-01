extern crate specs;
use specs::prelude::*;

extern crate rltk;
use rltk::{ Rltk, RGB };

use super::{ 
    ParticleLifetime,
    Position,
    Renderable, };

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: u8,
    lifetime: f32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder { requests: Vec::new() }
    }

    pub fn requests(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: u8, lifetime: f32) {
        self.requests.push(ParticleRequest { x, y, fg, bg, glyph, lifetime });
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, ParticleLifetime>,
                        WriteExpect<'a, ParticleBuilder> );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut positions, 
             mut renderables, 
             mut particles, 
             mut particle_builder) = data;

        for np in particle_builder.requests.iter() {
            let p = entities.create();
            positions.insert(p, Position { x: np.x, y: np.y }).expect("unable to inset position");
            renderables.insert(p, Renderable { fg: np.fg, bg: np.bg, glyph: np.glyph, render_order: 0 }).expect("unable to insert renderable");
            particles.insert(p, ParticleLifetime { lifetime_ms: np.lifetime }).expect("unable to insert lifetime");
        }
        
        particle_builder.requests.clear();
    }
}

pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles : Vec<Entity> = Vec::new();
    {
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }
    }

    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("particle will not die");
    }
}
