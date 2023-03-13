use pi_assets::{mgr::AssetMgr, asset::{GarbageEmpty, Handle}};
use pi_render::{renderer::vertex_buffer::{FixedSizeBufferPoolNotUpdatable, NotUpdatableBuffer, EVertexBufferRange, KeyVertexBuffer, VertexBufferAllocator}, rhi::{device::RenderDevice, RenderQueue}};
use pi_share::Share;

use crate::shaders::KeySpineShader;


pub struct SpineVertexBufferAllocator {
    // colored: FixedSizeBufferPoolNotUpdatable,
    // colored_textured: FixedSizeBufferPoolNotUpdatable,
    // two_colored_textured: FixedSizeBufferPoolNotUpdatable,
    // indices: FixedSizeBufferPoolNotUpdatable,
    // asset_mgr: Share<AssetMgr<NotUpdatableBuffer>>,
    counter: u32,
}
impl SpineVertexBufferAllocator {
    pub const MAX_VERTICES: u32 = 10920;
    pub fn init() -> Self {
        Self {
            counter: 0,
            // asset_mgr: AssetMgr::<NotUpdatableBuffer>::new(GarbageEmpty(), false, 2 * 1024 * 1024, 60 * 1000),
            // colored: FixedSizeBufferPoolNotUpdatable::new(Self::MAX_VERTICES * KeySpineShader::Colored.vertices_bytes_per_element(), wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX),
            // colored_textured: FixedSizeBufferPoolNotUpdatable::new(Self::MAX_VERTICES * KeySpineShader::ColoredTextured.vertices_bytes_per_element(), wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX),
            // two_colored_textured: FixedSizeBufferPoolNotUpdatable::new(Self::MAX_VERTICES * KeySpineShader::TwoColoredTextured.vertices_bytes_per_element(), wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX),
            // indices: FixedSizeBufferPoolNotUpdatable::new(Self::MAX_VERTICES * 3 * 2, wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX),
        }
    }
    pub fn colored(
        &mut self,
        vb_allocator: &mut VertexBufferAllocator,
        data: &[f32],
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Option<EVertexBufferRange> {
        if self.counter == u32::MAX {
            self.counter = 0;
        } else {
            self.counter += 1;
        }

        let key = KeyVertexBuffer::from(self.counter.to_string());
        if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
            Some(range)
        } else {
            None
        }
    }
    pub fn colored_textured(
        &mut self,
        vb_allocator: &mut VertexBufferAllocator,
        data: &[f32],
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Option<EVertexBufferRange> {
        if self.counter == u32::MAX {
            self.counter = 0;
        } else {
            self.counter += 1;
        }

        let key = KeyVertexBuffer::from(self.counter.to_string());
        if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
            Some(range)
        } else {
            None
        }
    }
    pub fn two_colored_textured(
        &mut self,
        vb_allocator: &mut VertexBufferAllocator,
        data: &[f32],
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Option<EVertexBufferRange> {
        if self.counter == u32::MAX {
            self.counter = 0;
        } else {
            self.counter += 1;
        }

        let key = KeyVertexBuffer::from(self.counter.to_string());
        if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
            Some(range)
        } else {
            None
        }
    }
    pub fn indices(
        &mut self,
        vb_allocator: &mut VertexBufferAllocator,
        data: &[u16],
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Option<EVertexBufferRange> {
        if self.counter == u32::MAX {
            self.counter = 0;
        } else {
            self.counter += 1;
        }

        let key = KeyVertexBuffer::from(self.counter.to_string());
        if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
            Some(range)
        } else {
            None
        }
    }
}