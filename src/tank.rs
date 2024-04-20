use bevy::prelude::*;

use crate::{
    physics::{Collider, Intersection, Rigidbody, WorldTransform},
    TILE_SIZE,
};

pub struct TankPlugin;

impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_tank)
            .add_systems(PostUpdate, add_texture_to_tanks);
    }
}

#[derive(Component, Debug)]
pub struct Tank;

#[derive(Bundle)]
pub struct TankBundle {
    pub tank: Tank,
    pub spatial_bundle: SpatialBundle,
    pub world_transform: WorldTransform,
    pub rigidbody: Rigidbody,
    pub collider: Collider,
    pub intersection: Intersection,
}

impl Default for TankBundle {
    fn default() -> Self {
        TankBundle {
            tank: Tank,
            spatial_bundle: default(),
            world_transform: default(),
            rigidbody: default(),
            collider: Collider {
                size: Vec2::new(8.0, 8.0),
            },
            intersection: default(),
        }
    }
}

fn render_tank(mut gizmos: Gizmos, query: Query<(&Transform, &Collider), With<Tank>>) {
    for (transform, collider) in query.iter() {
        gizmos.rect_2d(
            transform.translation.xy(),
            0.0,
            collider.size * TILE_SIZE,
            Color::WHITE,
        );
    }
}

fn add_texture_to_tanks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Collider), (Without<Handle<Image>>, With<Tank>)>,
) {
    for (entity, collider) in query.iter() {
        commands.entity(entity).insert((
            asset_server.load::<Image>("textures/tank.png"),
            Sprite {
                custom_size: Some(collider.size * TILE_SIZE),
                ..default()
            },
        ));
    }
}
