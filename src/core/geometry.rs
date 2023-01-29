use super::vertex::Vertex;

pub static TRI: [Vertex; 6] = [
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
    Vertex {
        color: cgmath::Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        pos: cgmath::Vector2 { x: -0.5, y: -0.5 },
    },
];
