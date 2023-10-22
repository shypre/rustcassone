use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

/// A right triangle on the `XY` plane centered at the right angle corner. If not flipped, right angle is pointing down and left, otherwise down and right.
#[derive(Debug, Copy, Clone)]
pub struct RightScaleneTriangle {
    /// Length of each right angle side.
    pub side_length_1: f32,
    pub side_length_2: f32,
    pub flipped: bool,
}

impl Default for RightScaleneTriangle {
    fn default() -> Self {
        RightScaleneTriangle::new(1.0, 1.0, false)
    }
}

impl RightScaleneTriangle {
    pub fn new(side_length_1: f32, side_length_2: f32, flipped: bool) -> Self {
        Self {
            side_length_1,
            side_length_2,
            flipped,
        }
    }
}

impl From<RightScaleneTriangle> for Mesh {
    fn from(right_triangle: RightScaleneTriangle) -> Self {
        let vertices: Vec<[f32; 3]>;
        let indices: Indices;

        if right_triangle.flipped {
            vertices = vec![
                [0.0, 0.0, 0.0],
                [0.0, right_triangle.side_length_1, 0.0],
                [-right_triangle.side_length_2, 0.0, 0.0],
            ];
            indices = Indices::U32(vec![0, 2, 1]);
        } else {
            vertices = vec![
                [0.0, 0.0, 0.0],
                [0.0, right_triangle.side_length_1, 0.0],
                [right_triangle.side_length_2, 0.0, 0.0],
            ];
            indices = Indices::U32(vec![0, 1, 2]);
        }

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

/// A right triangle on the `XY` plane centered at the right angle corner, both sides are of equal length.
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

/// SquareWithTrangleChunk, centered at square center
#[derive(Debug, Copy, Clone)]
pub struct SquareWithTrangleChunk {
    /// Length of square. Chunk height is 1/4 of this.
    pub side_length: f32,
}

impl Default for SquareWithTrangleChunk {
    fn default() -> Self {
        SquareWithTrangleChunk::new(1.0)
    }
}

impl SquareWithTrangleChunk {
    pub fn new(side_width: f32) -> Self {
        Self {
            side_length: side_width,
        }
    }
}

impl From<SquareWithTrangleChunk> for Mesh {
    fn from(square_with_chunk: SquareWithTrangleChunk) -> Self {
        let vertices = vec![
            [
                -square_with_chunk.side_length / 2.0,
                square_with_chunk.side_length / 2.0,
                0.0,
            ],
            [
                square_with_chunk.side_length / 2.0,
                square_with_chunk.side_length / 2.0,
                0.0,
            ],
            [
                -square_with_chunk.side_length / 2.0,
                -square_with_chunk.side_length / 2.0,
                0.0,
            ],
            [
                square_with_chunk.side_length / 2.0,
                -square_with_chunk.side_length / 2.0,
                0.0,
            ],
            [0.0, square_with_chunk.side_length / 4.0, 0.0],
        ];

        let indices = Indices::U32(vec![0, 4, 2, 4, 2, 3, 1, 3, 4]);

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

/// SquareWithTwoTrangleChunks, centered at square center
#[derive(Debug, Copy, Clone)]
pub struct SquareWithTwoTrangleChunks {
    /// Length of square. Chunk height is 1/4 of this.
    pub side_length: f32,
}

impl Default for SquareWithTwoTrangleChunks {
    fn default() -> Self {
        SquareWithTwoTrangleChunks::new(1.0)
    }
}

impl SquareWithTwoTrangleChunks {
    pub fn new(side_width: f32) -> Self {
        Self {
            side_length: side_width,
        }
    }
}

impl From<SquareWithTwoTrangleChunks> for Mesh {
    fn from(square_two_chunks: SquareWithTwoTrangleChunks) -> Self {
        let vertices = vec![
            [
                -square_two_chunks.side_length / 2.0,
                square_two_chunks.side_length / 2.0,
                0.0,
            ],
            [
                square_two_chunks.side_length / 2.0,
                square_two_chunks.side_length / 2.0,
                0.0,
            ],
            [
                -square_two_chunks.side_length / 2.0,
                -square_two_chunks.side_length / 2.0,
                0.0,
            ],
            [
                square_two_chunks.side_length / 2.0,
                -square_two_chunks.side_length / 2.0,
                0.0,
            ],
            [0.0, square_two_chunks.side_length / 4.0, 0.0],
            [0.0, -square_two_chunks.side_length / 4.0, 0.0],
        ];

        let indices = Indices::U32(vec![0, 4, 5, 0, 5, 2, 1, 5, 4, 1, 3, 5]);

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

/// SquashedTriangle, centered at hypotenuse right angles to vertex of obtuse angle
#[derive(Debug, Copy, Clone)]
pub struct SquashedTriangle {
    /// Length of triangle hypotenuse. Height is 1/4 of this.
    pub side_length: f32,
}

impl Default for SquashedTriangle {
    fn default() -> Self {
        SquashedTriangle::new(1.0)
    }
}

impl SquashedTriangle {
    pub fn new(side_width: f32) -> Self {
        Self {
            side_length: side_width,
        }
    }
}

impl From<SquashedTriangle> for Mesh {
    fn from(square_with_chunk: SquashedTriangle) -> Self {
        let vertices = vec![
            [-square_with_chunk.side_length / 2.0, 0.0, 0.0],
            [square_with_chunk.side_length / 2.0, 0.0, 0.0],
            [0.0, square_with_chunk.side_length / 4.0, 0.0],
        ];

        let indices = Indices::U32(vec![0, 2, 1]);

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
