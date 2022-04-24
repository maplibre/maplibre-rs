use std::vec;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    pub fn new_2d(position: cgmath::Vector2<f32>) -> Vertex {
        Vertex {
            position: position.extend(0.0).into(),
            uv: [0.0, 0.0],
        }
    }

    pub fn new_2d_uv(position: cgmath::Vector2<f32>, uv: cgmath::Vector2<f32>) -> Vertex {
        Vertex {
            position: position.extend(0.0).into(),
            uv: uv.into(),
        }
    }

    pub fn new_3d(position: cgmath::Vector3<f32>) -> Vertex {
        Vertex {
            position: position.into(),
            uv: [0.0, 0.0],
        }
    }

    pub fn new_3d_uv(position: cgmath::Vector3<f32>, uv: cgmath::Vector2<f32>) -> Vertex {
        Vertex {
            position: position.into(),
            uv: uv.into(),
        }
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
    pub center: cgmath::Vector3<f32>,
    pub width: f32,
    pub height: f32,
}

impl Meshable for Quad {
    fn as_mesh(&self, device: &wgpu::Device) -> Mesh {
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;
        let mut vertices = Vec::new();

        let top_left = self.center + cgmath::Vector3::new(-half_width, half_height, 0.0);
        let bottom_left = self.center + cgmath::Vector3::new(-half_width, -half_height, 0.0);
        let bottom_right = self.center + cgmath::Vector3::new(half_width, -half_height, 0.0);
        let top_right = self.center + cgmath::Vector3::new(half_width, half_height, 0.0);

        vertices.push(Vertex::new_3d(top_left));
        vertices.push(Vertex::new_3d(bottom_left));
        vertices.push(Vertex::new_3d(bottom_right));
        vertices.push(Vertex::new_3d(top_right));
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        let mesh = Mesh::new(vertices, indices, device);
        mesh
    }
}
