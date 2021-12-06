use crate::components::{Particle, Position, Renderable};
use rltk::{Rltk, RGB};
use specs::prelude::*;

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
}

pub struct RequestQueue {
    requests: Vec<ParticleRequest>,
}

impl RequestQueue {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            requests: Default::default(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        });
    }
    fn pop(&mut self) -> Option<ParticleRequest> {
        self.requests.pop()
    }
}

pub struct ParticleSpawnSystem;

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Particle>,
        WriteExpect<'a, RequestQueue>,
    );

    fn run(
        &mut self,
        (entities, mut positions, mut renderables, mut particles, mut requests): Self::SystemData,
    ) {
        while let Some(request) = requests.pop() {
            let p = entities.create();
            positions
                .insert(
                    p,
                    Position {
                        x: request.x,
                        y: request.y,
                    },
                )
                .expect("Failed to spawn particle");
            renderables
                .insert(
                    p,
                    Renderable {
                        glyph: request.glyph,
                        fg: request.fg,
                        bg: request.bg,
                        render_order: 3,
                    },
                )
                .expect("Failed to spawn particle");
            particles
                .insert(
                    p,
                    Particle {
                        lifetime: request.lifetime,
                    },
                )
                .expect("Failed to spawn particle");
        }
    }
}

pub fn particle_lifecycle(ecs: &mut World, ctx: &Rltk) {
    let mut dead = Vec::new();
    {
        let entities = ecs.entities();
        let mut particles = ecs.write_storage::<Particle>();
        for (e, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime -= ctx.frame_time_ms;
            if particle.lifetime < 0. {
                dead.push(e)
            }
        }
    }
    ecs.delete_entities(&dead)
        .expect("Failed to delete dead particles")
}
