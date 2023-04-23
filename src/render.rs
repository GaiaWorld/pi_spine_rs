use pi_hash::XHashMap;
use pi_render::rhi::device::RenderDevice;
use pi_spine_render::{shaders::{SingleSpineBindGroupLayout, SingleSpinePipelinePool}};

pub struct SpineResource {
    pub(crate) layouts: SingleSpineBindGroupLayout,
    pub(crate) pipelines: SingleSpinePipelinePool,
}
impl SpineResource {
    pub fn new(device: &RenderDevice) -> Self {
        let layouts = SingleSpineBindGroupLayout::new(device);
        let pipelines = SingleSpinePipelinePool::new(device);

        Self { layouts, pipelines }
    }
}

pub struct KeySpineRenderer(pub(crate) u32);