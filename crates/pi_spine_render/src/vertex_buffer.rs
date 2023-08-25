
use std::sync::Arc;

use pi_render::{renderer::vertex_buffer::{ EVertexBufferRange, KeyVertexBuffer, VertexBufferAllocator, NotUpdatableBufferRange}, rhi::{device::RenderDevice, RenderQueue}};

pub struct InstanceCacheBuffer {
    vertices: Vec<u8>,
    buffer: (Arc<NotUpdatableBufferRange>, u32, u32),
}

pub struct SpineVertexBufferAllocator {
    list: Vec<InstanceCacheBuffer>,
    used_index: usize,
    /// 单个 Mesh 的实例化最多使用多少字节数据
    /// 当 运行时超过该数据时 对数据进行截取
    one_mesh_max_instance_bytes: u32,
}
impl SpineVertexBufferAllocator {
    pub fn new(one_mesh_max_instance_bytes: u32) -> Self {
        Self {
            list: vec![],
            used_index: 0,
            one_mesh_max_instance_bytes,
        }
    }
    pub fn instance_initial_buffer(&self) -> (Arc<NotUpdatableBufferRange>, u32, u32) {
        (self.list[0].buffer.0.clone(), 0, 0)
    }
    /// 默认都是 f32
    pub fn collect(&mut self, data: &[u8], bytes_per_instance: u32, allocator: &mut VertexBufferAllocator, device: &RenderDevice, queue: &RenderQueue) -> Option<(Arc<NotUpdatableBufferRange>, u32, u32)> {
        let max_count = self.one_mesh_max_instance_bytes / bytes_per_instance;
        let byte_size = data.len().min((max_count * bytes_per_instance) as usize);
        let bytes = &data[0..byte_size];

        if let Some(buffer) = self.list.get(self.used_index) {
            if buffer.vertices.len() + bytes.len() > buffer.buffer.2 as usize {
                self.used_index += 1;
            }
        };
        if let Some(buffer) = self.list.get_mut(self.used_index)  {
            let start = buffer.vertices.len();
            bytes.iter().for_each(|v| { buffer.vertices.push(*v) });
            return Some((buffer.buffer.0.clone(), start as u32, buffer.vertices.len() as u32));
        } else {
            let mut data = Vec::with_capacity(self.one_mesh_max_instance_bytes as usize);
            bytes.iter().for_each(|v| { data.push(*v) });
            let vertices = data.clone();

            for _ in byte_size..self.one_mesh_max_instance_bytes as usize {
                data.push(0);
            }
            if let Some(buffer) = allocator.create_not_updatable_buffer_pre(device, queue, &data, None) {
                self.list.push(InstanceCacheBuffer {
                    vertices,
                    buffer: (buffer, 0, self.one_mesh_max_instance_bytes as u32),
                    // key: KeyVertexBuffer::from(self.used_index.to_string().as_str()),
                });
                let buffer = self.list.get_mut(self.used_index).unwrap();
                return Some(
                    (buffer.buffer.0.clone(), 0, buffer.vertices.len() as u32)
                );
            } else {
                return None;
            }
        };
    }
    pub fn upload(&mut self, queue: &RenderQueue) {
        for idx in 0..(self.used_index + 1) {
            if let Some(buffer) = self.list.get_mut(idx) {
                if buffer.vertices.len() > 0 {
                    queue.write_buffer(buffer.buffer.0.buffer(), 0, &buffer.vertices);
                }
                buffer.vertices.clear();
            }
        }
        self.used_index = 0;
    }
}

pub struct SpineIndicesBufferAllocator {
    list: Vec<InstanceCacheBuffer>,
    used_index: usize,
    /// 单个 Mesh 的实例化最多使用多少字节数据
    /// 当 运行时超过该数据时 对数据进行截取
    one_mesh_max_instance_bytes: u32,
}
impl SpineIndicesBufferAllocator {
    pub fn new(one_mesh_max_instance_bytes: u32) -> Self {
        Self {
            list: vec![],
            used_index: 0,
            one_mesh_max_instance_bytes,
        }
    }
    pub fn instance_initial_buffer(&self) -> (Arc<NotUpdatableBufferRange>, u32, u32) {
        (self.list[0].buffer.0.clone(), 0, 0)
    }
    /// 默认都是 f32
    pub fn collect(&mut self, data: &[u8], bytes_per_instance: u32, allocator: &mut VertexBufferAllocator, device: &RenderDevice, queue: &RenderQueue) -> Option<(Arc<NotUpdatableBufferRange>, u32, u32)> {
        let max_count = self.one_mesh_max_instance_bytes / bytes_per_instance;
        let byte_size = data.len().min((max_count * bytes_per_instance) as usize);
        let bytes = &data[0..byte_size];

        if let Some(buffer) = self.list.get(self.used_index) {
            if buffer.vertices.len() + bytes.len() > buffer.buffer.2 as usize {
                self.used_index += 1;
            }
        };
        if let Some(buffer) = self.list.get_mut(self.used_index)  {
            let start = buffer.vertices.len();
            bytes.iter().for_each(|v| { buffer.vertices.push(*v) });
            return Some((buffer.buffer.0.clone(), start as u32, buffer.vertices.len() as u32));
        } else {
            let mut data = Vec::with_capacity(self.one_mesh_max_instance_bytes as usize);
            bytes.iter().for_each(|v| { data.push(*v) });
            let vertices = data.clone();

            for _ in byte_size..self.one_mesh_max_instance_bytes as usize {
                data.push(0);
            }
            if let Some(buffer) = allocator.create_not_updatable_buffer_for_index(device, queue, &data) {
                match buffer {
                    EVertexBufferRange::Updatable(_, _, _) => { return None; },
                    EVertexBufferRange::NotUpdatable(buffer, _, _) => {
                        self.list.push(InstanceCacheBuffer {
                            vertices,
                            buffer: (buffer, 0, self.one_mesh_max_instance_bytes as u32),
                        });
                        let buffer = self.list.get_mut(self.used_index).unwrap();
                        return Some(
                            (buffer.buffer.0.clone(), 0, buffer.vertices.len() as u32)
                        );
                    },
                }
            } else {
                return None;
            }
        };
    }
    pub fn upload(&mut self, queue: &RenderQueue) {
        for idx in 0..(self.used_index + 1) {
            if let Some(buffer) = self.list.get_mut(idx) {
                let count = buffer.vertices.len() / 4;
                if count * 4 != buffer.vertices.len() {
                    let diff = (count + 1) * 4 - buffer.vertices.len();
                    for _ in 0..diff {
                        buffer.vertices.push(0);
                    }
                };

                if buffer.vertices.len() > 0 {
                    queue.write_buffer(buffer.buffer.0.buffer(), 0, &buffer.vertices);
                }
                buffer.vertices.clear();
            }
        }
        self.used_index = 0;
    }
}


// pub struct SpineVertexBufferAllocator;
// impl SpineVertexBufferAllocator {
//     pub const MAX_VERTICES: u32 = 10920;
//     pub fn init() -> Self {
//         Self
//     }
//     pub fn colored(
//         &mut self,
//         vb_allocator: &mut VertexBufferAllocator,
//         data: &[f32],
//         device: &RenderDevice,
//         queue: &RenderQueue,
//     ) -> Option<EVertexBufferRange> {
//         if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
//             Some(range)
//         } else {
//             None
//         }
//     }
//     pub fn colored_textured(
//         &mut self,
//         vb_allocator: &mut VertexBufferAllocator,
//         data: &[f32],
//         device: &RenderDevice,
//         queue: &RenderQueue,
//     ) -> Option<EVertexBufferRange> {
//         if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
//             Some(range)
//         } else {
//             None
//         }
//     }
//     pub fn two_colored_textured(
//         &mut self,
//         vb_allocator: &mut VertexBufferAllocator,
//         data: &[f32],
//         device: &RenderDevice,
//         queue: &RenderQueue,
//     ) -> Option<EVertexBufferRange> {
//         if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
//             Some(range)
//         } else {
//             None
//         }
//     }
//     pub fn indices(
//         &mut self,
//         vb_allocator: &mut VertexBufferAllocator,
//         data: &[u16],
//         device: &RenderDevice,
//         queue: &RenderQueue,
//     ) -> Option<EVertexBufferRange> {
//         if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
//             Some(range)
//         } else {
//             None
//         }
//     }
// }