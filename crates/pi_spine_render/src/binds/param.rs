use std::{num::NonZeroU64, sync::Arc, fmt::Debug};

use pi_assets::{asset::{Handle, Asset, GarbageEmpty, Size}, mgr::AssetMgr};
use pi_render::rhi::{device::RenderDevice, BufferInitDescriptor, RenderQueue, buffer::Buffer};
use pi_share::{Share, ShareMutex};

pub struct BindBufferAllocator {
    asset_mgr: Share<AssetMgr<SpineBindBuffer>>,
    couter: usize,
    list: Arc<Vec<usize>>,
    mutex: ShareMutex<()>,
}
impl BindBufferAllocator {
    pub fn new(capacity: usize, timeout: usize) -> Self {
        Self {
            asset_mgr: AssetMgr::<SpineBindBuffer>::new(GarbageEmpty(), false, capacity, timeout),
            couter: 0,
            list: Arc::new(vec![]),
            mutex: ShareMutex::new(())
        }
    }
    pub fn allocate(&mut self, device: &RenderDevice, queue: &RenderQueue, data: &[u8]) -> Option<SpineBindBufferUsage> {
        let list = unsafe {
            &mut *(Arc::as_ptr(&self.list) as usize as *mut Vec<usize>)
        };
        let idx = if let Some(idx) = list.pop() {
            idx
        } else {
            self.couter += 1;
            self.couter
        };

        if let Some(buffer) = self.asset_mgr.get(&idx) {
            queue.write_buffer(&buffer.buffer, 0, data);
            Some(SpineBindBufferUsage(buffer))
        } else {
            let buffer = device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: None,
                    contents: &[0;BindParam::SIZE],
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                }
            );
            let buffer = SpineBindBuffer {
                buffer,
                idx,
                list: self.list.clone(),
            };
            queue.write_buffer(&buffer.buffer, 0, data);
            if let Ok(buffer) = self.asset_mgr.insert(idx, buffer) {
                Some(SpineBindBufferUsage(buffer))
            } else {
                None
            }
        }
    }
}

pub struct SpineBindBuffer {
    buffer: Buffer,
    idx: usize,
    list: Arc<Vec<usize>>,
}
impl SpineBindBuffer {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
impl Debug for SpineBindBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpineBindBuffer").field("idx", &self.idx).finish()
    }
}
impl Asset for SpineBindBuffer {
    type Key = usize;
}

impl Size for SpineBindBuffer {
    fn size(&self) -> usize {
        BindParam::SIZE
    }
}

#[derive(Debug, Clone)]
pub struct SpineBindBufferUsage(pub Handle<SpineBindBuffer>);
impl Drop for SpineBindBufferUsage {
    fn drop(&mut self) {
        let list = unsafe {
            &mut *(Arc::as_ptr(&self.0.list) as usize as *mut Vec<usize>)
        };
        list.push(self.0.idx)
    }
}

pub struct BindParam {
    pub(crate) buffer: Handle<SpineBindBuffer>,
}
impl BindParam {
    pub const SIZE: usize = (16 + 4 + 4) * 4;
    pub fn layout_entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: NonZeroU64::new(Self::SIZE as u64) },
            count: None,
        }
    }
}