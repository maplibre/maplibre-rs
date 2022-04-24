use std::vec;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3 + 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    pub fn new_2d(x: f32, y: f32) -> Vertex {
        Vertex {
            position: [x, y, 0.0],
            color: [0.0, 0.0, 0.0, 0.2],
            uv: [0.0, 0.0],
        }
    }

    pub fn new_2d_uv(x: f32, y: f32, u: f32, v: f32) -> Vertex {
        Vertex {
            position: [x, y, 0.0],
            color: [0.0, 0.0, 0.0, 0.2],
            uv: [u, v],
        }
    }

    pub fn new_3d(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, a: f32) -> Vertex {
        Vertex {
            position: [x, y, z],
            color: [r, g, b, a],
            uv: [0.0, 0.0],
        }
    }

    pub fn x(&self) -> f32 {
        self.position[0]
    }

    pub fn y(&self) -> f32 {
        self.position[1]
    }

    pub fn z(&self) -> f32 {
        self.position[2]
    }

    pub fn r(&self) -> f32 {
        self.color[0]
    }

    pub fn g(&self) -> f32 {
        self.color[1]
    }

    pub fn b(&self) -> f32 {
        self.color[2]
    }

    pub fn a(&self) -> f32 {
        self.color[3]
    }
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: usize,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>, device: &wgpu::Device) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TestVertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_indices = indices.len();

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TestIndices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Mesh {
            // Take ownership of the data underlying the buffers!
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
}

pub trait Meshable {
    fn as_mesh(&self, device: &wgpu::Device) -> Mesh;
}

pub struct Quad {
    pub center: Vertex,
    pub width: f32,
    pub height: f32,
}

impl Meshable for Quad {
    fn as_mesh(&self, device: &wgpu::Device) -> Mesh {
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;
        let mut vertices = Vec::new();
        vertices.push(Vertex::new_3d(
            self.center.x() - half_width,
            self.center.y() + half_height,
            self.center.z(),
            self.center.r(),
            self.center.g(),
            self.center.b(),
            self.center.a(),
        ));
        vertices.push(Vertex::new_3d(
            self.center.x() - half_width,
            self.center.y() - half_height,
            self.center.z(),
            self.center.r(),
            self.center.g(),
            self.center.b(),
            self.center.a(),
        ));
        vertices.push(Vertex::new_3d(
            self.center.x() + half_width,
            self.center.y() - half_height,
            self.center.z(),
            self.center.r(),
            self.center.g(),
            self.center.b(),
            self.center.a(),
        ));
        vertices.push(Vertex::new_3d(
            self.center.x() + half_width,
            self.center.y() + half_height,
            self.center.z(),
            self.center.r(),
            self.center.g(),
            self.center.b(),
            self.center.a(),
        ));
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        let mesh = Mesh::new(vertices, indices, device);
        mesh
    }
}
