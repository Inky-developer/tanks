use std::cmp::Ordering;

use bevy::prelude::*;

/// Calculates the shortest vector from the line to the point
/// The line is assumed to originate at (0|0) and to go to the point defined by `line`
pub fn vector_line_point(line: Vec2, point: Vec2) -> Vec2 {
    let vec_origin_point = point;
    let vec_line_end_point = point - line;

    if line == Vec2::ZERO {
        return vec_origin_point;
    }

    // t is the point on the line defined as `line * t` with the smallest distance to the target point
    let t = (line.x * point.x + line.y * point.y) / (line.x.powi(2) + line.y.powi(2));
    if t <= 0.0 || t >= 1.0 {
        return min_by_key(vec_origin_point, vec_line_end_point, |vector| {
            vector.length()
        });
    }
    point - (line * t)
}

pub fn min_by_key<T, U, F>(a: T, b: T, mut function: F) -> T
where
    F: FnMut(&T) -> U,
    U: PartialOrd,
{
    let val_a = function(&a);
    let val_b = function(&b);
    match val_a.partial_cmp(&val_b) {
        Some(Ordering::Less | Ordering::Equal) | None => a,
        Some(Ordering::Greater) => b,
    }
}

pub fn max_by_key<T, U, F>(a: T, b: T, mut function: F) -> T
where
    F: FnMut(&T) -> U,
    U: PartialOrd,
{
    let val_a = function(&a);
    let val_b = function(&b);
    match val_a.partial_cmp(&val_b) {
        Some(Ordering::Greater | Ordering::Equal) | None => a,
        Some(Ordering::Less) => b,
    }
}
