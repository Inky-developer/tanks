use bevy::math::Vec2;

use self::world_gen::Wave;

#[derive(Debug)]
pub struct World {
    pub width: usize,
    pub height: usize,
    data: Vec<WorldTile>,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum WorldTile {
    #[default]
    Air,
    Dirt,
}

impl WorldTile {
    pub fn is_not_air(self) -> bool {
        !matches!(self, Self::Air)
    }
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![WorldTile::default(); width * height],
        }
    }

    pub fn generate(width: usize, height: usize) -> Self {
        let w = width as f32;
        let h = height as f32;
        let worldgen_config = [
            Wave {
                height: h * 0.25,
                speed: std::f32::consts::PI / w,
                off_y: h * 0.3,
                off_x: 0.0,
            },
            Wave {
                height: h * 0.1,
                speed: std::f32::consts::PI / w * 4.0,
                off_y: 0.0,
                off_x: std::f32::consts::PI,
            },
            Wave {
                height: h * 0.015,
                speed: 0.3,
                off_y: 0.0,
                off_x: 42.0,
            },
        ];
        world_gen::generate_world(width, height, &worldgen_config)
    }

    pub fn fill_radius(
        &mut self,
        explosion_x: isize,
        explosion_y: isize,
        radius: f32,
        tile: WorldTile,
    ) {
        let explosion_pos = Vec2::new(explosion_x as f32, explosion_y as f32);
        let radius_int = (radius + 0.5) as isize;
        for x in (explosion_x - radius_int)..(explosion_x + radius_int) {
            if x < 0 || x >= self.width as isize {
                continue;
            }
            for y in (explosion_y - radius_int)..(explosion_y + radius_int) {
                if y < 0 || y >= self.height as isize {
                    continue;
                }

                let pos = Vec2::new(x as f32, y as f32);
                let distance = pos.distance(explosion_pos);
                if distance <= radius {
                    self.set(x, y, tile);
                }
            }
        }
    }

    pub fn set(&mut self, x: isize, y: isize, tile: WorldTile) {
        let idx = self.coords_to_index(x, y);
        self.data[idx] = tile;
    }

    pub fn get(&self, x: isize, y: isize) -> WorldTile {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            return WorldTile::default();
        }
        let idx = self.coords_to_index(x, y);
        self.data[idx]
    }

    fn coords_to_index(&self, x: isize, y: isize) -> usize {
        assert!(x < self.width as isize && x >= 0);
        assert!(y < self.height as isize && y >= 0);
        let idx = x + y * self.width as isize;
        idx as usize
    }
}

mod world_gen {
    use super::World;

    #[derive(Debug, Clone, Copy)]
    pub(super) struct Wave {
        pub height: f32,
        pub speed: f32,
        pub off_y: f32,
        pub off_x: f32,
    }

    impl Wave {
        fn at_x(&self, x: f32) -> f32 {
            self.off_y + ((x + self.off_x) * self.speed).sin() * self.height
        }
    }

    pub(super) fn generate_world(width: usize, height: usize, waves: &[Wave]) -> World {
        let mut world = World::new(width, height);
        for x in 0..world.width {
            let height: f32 = waves.iter().map(|wave| wave.at_x(x as f32)).sum();
            for y in 0..(usize::min(world.height, height as usize)) {
                world.set(x as isize, y as isize, super::WorldTile::Dirt);
            }
        }

        world
    }
}
