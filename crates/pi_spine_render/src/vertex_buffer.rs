
use pi_render::{renderer::vertex_buffer::{ EVertexBufferRange, KeyVertexBuffer, VertexBufferAllocator}, rhi::{device::RenderDevice, RenderQueue}};



pub struct SpineVertexBufferAllocator;
impl SpineVertexBufferAllocator {
    pub const MAX_VERTICES: u32 = 10920;
    pub fn init() -> Self {
        Self
    }
    pub fn colored(
        &mut self,
        vb_allocator: &mut VertexBufferAllocator,
        data: &[f32],
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Option<EVertexBufferRange> {
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
        if let Some(range) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(data)) {
            Some(range)
        } else {
            None
        }
    }
}