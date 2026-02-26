//! Math utilities for the editor.

/// Clamp a value between min and max.
#[inline]
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Linear interpolation between two values.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Calculate the absolute difference between two values.
#[inline]
pub fn abs_diff<T: PartialOrd + std::ops::Sub<Output = T>>(a: T, b: T) -> T {
    if a > b {
        a - b
    } else {
        b - a
    }
}

/// Check if a value is approximately equal to another within epsilon.
#[inline]
pub fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

/// Calculate the distance between two points.
#[inline]
pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

/// Calculate the Manhattan distance between two points.
#[inline]
pub fn manhattan_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x2 - x1).abs() + (y2 - y1).abs()
}

/// Calculate the Chebyshev distance (max of absolute differences).
#[inline]
pub fn chebyshev_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x2 - x1).abs().max((y2 - y1).abs())
}

/// Round a value to the nearest multiple.
#[inline]
pub fn round_to_multiple(value: f64, multiple: f64) -> f64 {
    (value / multiple).round() * multiple
}

/// Floor a value to the nearest multiple.
#[inline]
pub fn floor_to_multiple(value: f64, multiple: f64) -> f64 {
    (value / multiple).floor() * multiple
}

/// Sign of a number (-1, 0, or 1).
#[inline]
pub fn signum<T: PartialOrd + Default>(value: T) -> i32
where
    T: std::ops::Neg<Output = T>,
{
    if value > T::default() {
        1
    } else if value < T::default() {
        -1
    } else {
        0
    }
}

/// A 2D point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
}

impl Point {
    /// Create a new point at the given coordinates.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Calculate Euclidean distance to another point.
    pub fn distance_to(&self, other: &Point) -> f64 {
        distance(self.x as f64, self.y as f64, other.x as f64, other.y as f64)
    }

    /// Calculate Manhattan distance to another point.
    pub fn manhattan_distance_to(&self, other: &Point) -> i32 {
        manhattan_distance(self.x, self.y, other.x, other.y)
    }
}

/// A 2D rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// X coordinate of top-left corner.
    pub x: i32,
    /// Y coordinate of top-left corner.
    pub y: i32,
    /// Width of the rectangle.
    pub width: i32,
    /// Height of the rectangle.
    pub height: i32,
}

impl Rect {
    /// Create a new rectangle with the given position and dimensions.
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle from two corner points.
    pub fn from_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self {
            x: x1.min(x2),
            y: y1.min(y2),
            width: (x2 - x1).abs() + 1,
            height: (y2 - y1).abs() + 1,
        }
    }

    /// Get the left edge X coordinate.
    pub fn left(&self) -> i32 {
        self.x
    }

    /// Get the right edge X coordinate.
    pub fn right(&self) -> i32 {
        self.x + self.width - 1
    }

    /// Get the top edge Y coordinate.
    pub fn top(&self) -> i32 {
        self.y
    }

    /// Get the bottom edge Y coordinate.
    pub fn bottom(&self) -> i32 {
        self.y + self.height - 1
    }

    /// Check if a point is inside this rectangle.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left() && x <= self.right() && y >= self.top() && y <= self.bottom()
    }

    /// Check if this rectangle intersects another.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Get the area of this rectangle.
    pub fn area(&self) -> i32 {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_distance() {
        let d = distance(0.0, 0.0, 3.0, 4.0);
        assert!((d - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_point() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);

        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
        assert_eq!(p1.manhattan_distance_to(&p2), 7);
    }

    #[test]
    fn test_rect() {
        let rect = Rect::from_points(2, 2, 5, 5);

        assert_eq!(rect.width, 4);
        assert_eq!(rect.height, 4);
        assert!(rect.contains(3, 3));
        assert!(!rect.contains(1, 1));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0, 0, 5, 5);
        let r2 = Rect::new(3, 3, 5, 5);
        let r3 = Rect::new(10, 10, 5, 5);

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }
}
