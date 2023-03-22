use pi_hash::XHashMap;
use pi_render::rhi::device::RenderDevice;
use pi_spine_render::{renderer::Renderer, shaders::{SingleSpineBindGroupLayout, SingleSpinePipelinePool}};

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

pub struct SpineRenderGroup {
    counter: u32,
    list: XHashMap<u32, Renderer>,
}
impl SpineRenderGroup {
    pub fn new() -> Self {
        Self { counter: 0, list: XHashMap::default() }
    }
    pub fn create_renderer(&mut self) -> KeySpineRenderer {
        self.counter += 1;
        let id = self.counter;

        let render = Renderer::new();
        self.list.insert(id, render);

        KeySpineRenderer(id)
    }
    pub fn get_renderer(
        &mut self, key: &KeySpineRenderer
    ) -> &mut Renderer {
        self.list.get_mut(&key.0).unwrap()
    }
}