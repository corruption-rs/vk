use super::vertex::Vertex;

pub static QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        color: cgmath::Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        pos: cgmath::Vector2 { x: -0.5, y: -0.5 },
    },
    Vertex {
        color: cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        pos: cgmath::Vector2 { x: 0.5, y: -0.5 },
    },
    Vertex {
        color: cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        pos: cgmath::Vector2 { x: 0.5, y: 0.5 },
    },
    Vertex {
        color: cgmath::Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        pos: cgmath::Vector2 { x: -0.5, y: 0.5 },
    },
];

pub static QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
