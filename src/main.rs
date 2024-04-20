mod debug_plugin;
mod math;
mod physics;
mod tank;
mod world;
mod world_mesh;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_asset::RenderAssetUsages,
        render_resource::{PrimitiveTopology, VertexFormat},
    },
    sprite::Mesh2dHandle,
};
use debug_plugin::DebugPlugin;
use physics::PhysicsPlugin;
use tank::{TankBundle, TankPlugin};
use world::World;
use world_mesh::{WorldMesh2d, WorldMeshPlugin};

#[derive(Resource)]
pub struct WorldMesh(Mesh2dHandle);

#[derive(Resource, Deref, DerefMut)]
pub struct GameWorld(World);

fn main() {
    let window_resolution = (WIDTH as f32 * TILE_SIZE, HEIGHT as f32 * TILE_SIZE).into();
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: window_resolution,
                    ..default()
                }),
                ..default()
            }),
            WorldMeshPlugin,
            PhysicsPlugin,
            TankPlugin,
            DebugPlugin,
        ))
        .insert_resource(GameWorld(World::generate(WIDTH, HEIGHT)))
        .insert_resource(ClearColor(Color::rgb(0.5, 0.8, 0.99)))
        .add_systems(Startup, setup)
        .add_systems(Update, update_world_mesh)
        .run();
}

const WIDTH: usize = 200;
const HEIGHT: usize = 100;
const TILE_SIZE: f32 = 8.0;

fn setup(mut commands: Commands, meshes: Res<Assets<Mesh>>) {
    let world_mesh_handle = Mesh2dHandle(meshes.reserve_handle());
    commands.insert_resource(WorldMesh(world_mesh_handle.clone()));
    commands.spawn((
        WorldMesh2d,
        world_mesh_handle,
        // This bundle's components are needed for something to be rendered
        SpatialBundle::INHERITED_IDENTITY,
    ));

    commands.spawn(TankBundle {
        spatial_bundle: SpatialBundle {
            transform: Transform::from_xyz(250.0, 250.0, 0.0),
            ..default()
        },
        ..default()
    });

    // Spawn the camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(
            WIDTH as f32 / 2.0 * TILE_SIZE,
            HEIGHT as f32 / 2.0 * TILE_SIZE,
            0.0,
        )),
        ..default()
    });
}

/// This system updates the world mesh whenever the world has changed
fn update_world_mesh(
    world: Res<GameWorld>,
    world_mesh_handle: Option<Res<WorldMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !world.is_changed() {
        return;
    }

    trace!("World has changed!");

    let Some(world_mesh_handle) = world_mesh_handle else {
        return;
    };

    let mesh = gen_world_mesh(&world.0);
    meshes.insert(&world_mesh_handle.0 .0, mesh);
}

/// Builds a mesh from the world
fn gen_world_mesh(world: &World) -> Mesh {
    let mut world_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let mut v_pos = Vec::with_capacity(WIDTH * HEIGHT * 4);
    let mut v_local_pos = Vec::with_capacity(WIDTH * HEIGHT * 4);
    let mut v_color = Vec::with_capacity(WIDTH * HEIGHT * 4);
    let mut v_neighbors = Vec::with_capacity(WIDTH * HEIGHT * 4);
    let mut indices = Vec::with_capacity(WIDTH * HEIGHT * 6);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cx = x as f32 * TILE_SIZE;
            let cy = y as f32 * TILE_SIZE;
            let nx = (x + 1) as f32 * TILE_SIZE;
            let ny = (y + 1) as f32 * TILE_SIZE;
            let index = v_pos.len() as u32;
            v_pos.extend([[cx, cy, 0.0], [nx, cy, 0.0], [nx, ny, 0.0], [cx, ny, 0.0]]);
            v_local_pos.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            indices.extend([index, index + 1, index + 2, index + 2, index + 3, index]);
            v_color.extend([Color::GOLD.as_linear_rgba_u32(); 4]);

            let top = world.get(x as isize, y as isize + 1).is_not_air() as u32;
            let left = world.get(x as isize - 1, y as isize).is_not_air() as u32;
            let bottom = world.get(x as isize, y as isize - 1).is_not_air() as u32;
            let right = world.get(x as isize + 1, y as isize).is_not_air() as u32;
            let self_on = world.get(x as isize, y as isize).is_not_air() as u32;
            let neighbors_bitset = top | left << 1 | bottom << 2 | right << 3 | self_on << 4;
            v_neighbors.extend([neighbors_bitset; 4]);
        }
    }

    // Set the position attribute
    world_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    // And a RGB color attribute as well
    world_mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Uint32),
        v_color,
    );
    world_mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_LocalPos", 2, VertexFormat::Float32x2),
        v_local_pos,
    );
    world_mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Neighbors", 3, VertexFormat::Uint32),
        v_neighbors,
    );
    world_mesh.insert_indices(Indices::U32(indices));

    world_mesh
}
