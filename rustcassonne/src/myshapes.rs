use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

/// A right triangle on the `XY` plane centered at the right angle corner.
#[derive(Debug, Copy, Clone)]
pub struct RightTriangle {
    /// Length of each right angle side.
    pub side_length: f32,
}

impl Default for RightTriangle {
    fn default() -> Self {
        RightTriangle::new(1.0)
    }
}

impl RightTriangle {
    pub fn new(side_length: f32) -> Self {
        Self { side_length }
    }
}

impl From<RightTriangle> for Mesh {
    fn from(right_triangle: RightTriangle) -> Self {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [0.0, right_triangle.side_length, 0.0],
            [right_triangle.side_length, 0.0, 0.0],
        ];

        let indices = Indices::U32(vec![0, 1, 2]);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 0.0, 1.0]; vertices.len()],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        return mesh;
    }
}

/// A right triangle plus trapezoid along the hypotenuse on the `XY` plane centered at the right angle corner.
#[derive(Debug, Copy, Clone)]
pub struct RightTriangleAndTrapezoid {
    /// Length of each right angle side.
    pub side_length: f32,
    /// trapezoid width
    pub trapezoid_width: f32,
}

impl Default for RightTriangleAndTrapezoid {
    fn default() -> Self {
        RightTriangleAndTrapezoid::new(1.0, 0.5)
    }
}

impl RightTriangleAndTrapezoid {
    pub fn new(side_length: f32, trapezoid_width: f32) -> Self {
        Self {
            side_length,
            trapezoid_width,
        }
    }
}

impl From<RightTriangleAndTrapezoid> for Mesh {
    fn from(right_triangle_and_trapezoid: RightTriangleAndTrapezoid) -> Self {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [0.0, right_triangle_and_trapezoid.side_length, 0.0],
            [right_triangle_and_trapezoid.side_length, 0.0, 0.0],
            [
                right_triangle_and_trapezoid.side_length,
                right_triangle_and_trapezoid.trapezoid_width,
                0.0,
            ],
            [
                right_triangle_and_trapezoid.trapezoid_width,
                right_triangle_and_trapezoid.side_length,
                0.0,
            ],
        ];

        let indices = Indices::U32(vec![0, 1, 2, 1, 3, 2, 1, 4, 3]);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 0.0, 1.0]; vertices.len()],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        return mesh;
    }
}




/// A trapezoid on the `XY` plane centered at the small right angle corner.
#[derive(Debug, Copy, Clone)]
pub struct Trapezoid {
    /// Width of each side of the hypothetical right triangle.
    pub side_length: f32,
    /// trapezoid width
    pub trapezoid_width: f32,
}

impl Default for Trapezoid {
    fn default() -> Self {
        Trapezoid::new(1.0, 0.5)
    }
}

impl Trapezoid {
    pub fn new(side_width: f32, trapezoid_width: f32) -> Self {
        Self {
            side_length: side_width,
            trapezoid_width,
        }
    }
}

impl From<Trapezoid> for Mesh {
    fn from(trapezoid: Trapezoid) -> Self {
        let vertices = vec![
            [0.0, trapezoid.side_length - trapezoid.trapezoid_width, 0.0],
            [0.0, trapezoid.side_length, 0.0],
            [trapezoid.side_length - trapezoid.trapezoid_width, 0.0, 0.0],
            [trapezoid.side_length, 0.0, 0.0],
        ];

        let indices = Indices::U32(vec![0, 1, 2, 2, 1, 3]);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 0.0, 1.0]; vertices.len()],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        return mesh;
    }
}