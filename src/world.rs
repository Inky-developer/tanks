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
        let mut world = Self::new(width, height);
        for x in 0..world.width {
            let height = ((x as f32 * 0.05).cos() * height as f32) as usize;
            for y in 0..(usize::min(world.height, height)) {
                // for y in 0..x {
                world.set(x as isize, y as isize, WorldTile::Dirt);
            }
        }

        world
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
