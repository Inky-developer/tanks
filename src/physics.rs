use bevy::prelude::*;

use crate::{
    math::{self, max_by_key},
    GameWorld, TILE_SIZE,
};

/// Component that moves entities every physics step
#[derive(Component, Debug, Default)]
pub struct Rigidbody {
    pub motion: Vec2,
}

/// Entities with this component and a [`Intersection`] and ['GlobalTransform`] component will have collision detection
#[derive(Component, Debug)]
pub struct Collider;

/// Describes the current intersection of the entity with the world
#[derive(Component, Debug, Default)]
pub struct Intersection {
    pub correction: Vec2,
}

/// A transform in world coordinates
#[derive(Component, Debug, Default)]
pub struct WorldTransform {
    pub translation: Vec2,
    pub tile_position: (isize, isize),
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            ((
                apply_motion,
                (set_world_transform, reset_intersections),
                collide_with_world,
                apply_corrections,
            )
                .chain(),),
        );
    }
}

fn set_world_transform(mut query: Query<(&Transform, &mut WorldTransform)>) {
    for (transform, mut world_transform) in query.iter_mut() {
        let mut world_pos = transform.translation.xy();
        world_pos /= TILE_SIZE;
        *world_transform = WorldTransform {
            translation: world_pos,
            tile_position: (world_pos.x as isize, world_pos.y as isize),
        }
    }
}

fn apply_motion(time: Res<Time>, mut query: Query<(&mut Transform, &Rigidbody)>) {
    for (mut transform, body) in query.iter_mut() {
        transform.translation.x += body.motion.x * TILE_SIZE * time.delta_seconds();
        transform.translation.y += body.motion.y * TILE_SIZE * time.delta_seconds();
    }
}

fn apply_corrections(mut query: Query<(&mut Transform, &Intersection)>) {
    for (mut transform, intersection) in query.iter_mut() {
        if intersection.correction == Vec2::ZERO {
            continue;
        }
        transform.translation.x += intersection.correction.x * TILE_SIZE;
        transform.translation.y += intersection.correction.y * TILE_SIZE;
    }
}

fn reset_intersections(mut intersections: Query<&mut Intersection>) {
    for mut intersection in intersections.iter_mut() {
        *intersection = Intersection::default();
    }
}

fn collide_with_world(
    world: Res<GameWorld>,
    mut query: Query<(&WorldTransform, &Transform, &mut Intersection), With<Collider>>,
    mut gizmos: Gizmos,
) {
    for (world_transform, transform, mut intersection) in query.iter_mut() {
        let collider_rect =
            Rect::from_center_size(world_transform.translation, transform.scale.xy());

        let mut max_correction = Vec2::ZERO;
        let possible_collisions = world.get_rendered_in_rect(collider_rect);
        for world_tile in possible_collisions {
            if !world_tile.tile.has_collider() {
                continue;
            }

            let lines = get_tile_lines(world_tile.pos.0, world_tile.pos.1);
            for line in lines {
                if let Some(correction) = line.collide_rect(collider_rect) {
                    gizmos.line_2d(line.start * TILE_SIZE, line.end * TILE_SIZE, Color::RED);
                    max_correction =
                        max_by_key(max_correction, correction, |vector| vector.length());
                }
            }
        }

        if max_correction != Vec2::ZERO {
            intersection.correction = max_correction;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Line {
    pub start: Vec2,
    pub end: Vec2,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Line { start, end }
    }

    /// Tests for a collision and returns the correction vector is there is some
    pub fn collide_rect(self, rect: Rect) -> Option<Vec2> {
        let rect_center = rect.center();
        let distance_vector =
            math::vector_line_point(self.end - self.start, rect_center - self.start);
        let delta = rect.half_size() - distance_vector.abs();
        if delta.x > 0.0 && delta.y > 0.0 {
            let perp = self.dir().perp();
            // Determine the smallest amount the rect has to be moved to not collide anymore
            let factor = match perp {
                Vec2 { x, y } if x == 0.0 => delta.y / y,
                Vec2 { x, y } if y == 0.0 => delta.x / x,
                perp => (delta / perp).min_element(),
            };
            return Some(factor * perp);
        }
        None
    }

    /// The direction vector of this line
    pub fn dir(self) -> Vec2 {
        self.end - self.start
    }
}

fn get_tile_lines(x: isize, y: isize) -> [Line; 1] {
    let x = x as f32;
    let y = y as f32;
    [Line::new(
        Vec2::new(x, y + 1.0),
        Vec2::new(x + 1.0, y + 1.0),
    )]
}
