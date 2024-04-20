use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

use crate::{
    physics::{Collider, Rigidbody},
    world::{World, WorldTile},
    GameWorld, TILE_SIZE,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (move_box, input, show_cursor_selection))
            .add_systems(FixedUpdate, let_box_fall);
    }
}

fn setup() {}

fn let_box_fall(time: Res<Time>, mut colliders: Query<&mut Transform, With<Collider>>) {
    for mut transform in colliders.iter_mut() {
        transform.translation.y -= time.delta_seconds() * 4.0 * TILE_SIZE;
    }
}

fn move_box(input: Res<ButtonInput<KeyCode>>, mut bodies: Query<&mut Rigidbody>) {
    let mut motion_vec = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowRight) {
        motion_vec.x += 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        motion_vec.x -= 1.0
    }
    if input.pressed(KeyCode::ArrowUp) {
        motion_vec.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        motion_vec.y -= 1.0
    }

    motion_vec *= 10.0;
    for mut body in bodies.iter_mut() {
        body.motion = motion_vec;
    }
}

#[derive(Debug, Clone, Copy)]
struct WorldAction {
    pub kind: WorldActionKind,
    pub power: f32,
}

impl WorldAction {
    pub fn perform(&self, world: &mut World, x: isize, y: isize) {
        self.kind.perform(world, x, y, self.power)
    }

    pub fn next(self) -> Self {
        Self {
            kind: self.kind.next(),
            ..self
        }
    }
}

impl Default for WorldAction {
    fn default() -> Self {
        Self {
            kind: WorldActionKind::default(),
            power: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum WorldActionKind {
    PlaceAir,
    #[default]
    PlaceTile,
}

impl WorldActionKind {
    fn perform(&self, world: &mut World, x: isize, y: isize, power: f32) {
        use WorldActionKind::*;

        match self {
            PlaceAir => world.fill_radius(x, y, power, WorldTile::Air),
            PlaceTile => world.fill_radius(x, y, power, WorldTile::Dirt),
        }
    }

    fn next(self) -> Self {
        use WorldActionKind::*;
        match self {
            PlaceAir => PlaceTile,
            PlaceTile => PlaceAir,
        }
    }
}

/// This system allows users to modify the world
fn input(
    mut action: Local<WorldAction>,
    mut world: ResMut<GameWorld>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut scroll_event_reader: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        *action = action.next();
        info!("Switched action to {action:?}");
    }
    for event in scroll_event_reader.read() {
        action.power = f32::max(action.power + event.y, 1.0);
    }
    if buttons.pressed(MouseButton::Left) {
        let window = windows.single();
        if let Some(mouse_pos) = window.cursor_position() {
            let (x, y) = screen_coords_to_world(mouse_pos, window.height());
            action.perform(&mut world.0, x, y);
        }
    }
}

/// This system shows a debug outline around the currently selected block
fn show_cursor_selection(mut gizmos: Gizmos, windows: Query<&Window, With<PrimaryWindow>>) {
    let window = windows.single();
    if let Some(mouse_pos) = window.cursor_position() {
        let (x, y) = screen_coords_to_world(mouse_pos, window.height());
        gizmos.rect_2d(
            Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * TILE_SIZE,
            0.,
            Vec2::splat(TILE_SIZE),
            Color::RED,
        )
    }
}

fn screen_coords_to_world(mut pos: Vec2, height: f32) -> (isize, isize) {
    pos.y = height - pos.y;
    pos /= TILE_SIZE;
    (pos.x as isize, pos.y as isize)
}
