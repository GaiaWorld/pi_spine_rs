use std::{ops::Range, sync::Arc};

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_map::vecmap::VecMap;
use pi_render::{renderer::{draw_obj::{DrawObj, DrawBindGroups, DrawBindGroup}, sampler::SamplerRes, pipeline::KeyRenderPipelineState, vertices::{RenderVertices, EVerticesBufferUsage, RenderIndices}, draw_obj_list::DrawList, vertex_buffer::VertexBufferAllocator}, rhi::{asset::{TextureRes, RenderRes}, device::RenderDevice, RenderQueue, bind_group::BindGroup, PrimitiveState}};
use pi_share::Share;

use crate::{shaders::{KeySpineShader, KeySpinePipeline, SingleSpinePipelinePool, SingleSpineBindGroupLayout}, vertex_buffer::SpineVertexBufferAllocator, EPrimitive, binds::param::{SpineBindBuffer, BindBufferAllocator, SpineBindBufferUsage}, bind_groups::SpineBindGroup};




pub struct Renderer {
    pub(crate) binds: Vec<SpineBindBufferUsage>,
    pub(crate) bind_groups: Vec<SpineBindGroup>,
    pub(crate) draws: DrawList,
    pub(crate) shader: Option<KeySpineShader>,
    pub(crate) blend: wgpu::BlendState,
    pub(crate) bind: Option<SpineBindBufferUsage>,
    pub(crate) enableblend: bool,
}
impl Renderer {
    pub fn new() -> Self {
        Self {
            binds: vec![],
            bind_groups: vec![],
            draws: DrawList::default(),
            shader: None,
            blend: wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            bind: None,
            enableblend: false,
        }
    }
    pub fn drawlist(&self) -> &DrawList {
        &self.draws
    }
    pub fn reset(&mut self) {
        self.bind_groups.clear();
        self.binds.clear();
        self.draws.list.clear();
    }
    pub fn viewport(&mut self, viewport: &[f32]) {
        //
    }
    
    pub fn shader(
        &mut self,
        shader: Option<KeySpineShader>,
    ) {
        self.shader = shader;
    }

    pub fn uniform(
        &mut self,
        uniform_param: &[f32],
        allocator: &mut BindBufferAllocator,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) {
        let bind = allocator.allocate(device, queue, bytemuck::cast_slice(uniform_param));
        if let Some(bind) = &bind {
            self.binds.push(bind.clone());
        }
        self.bind = bind;
    }
    pub fn blend(&mut self, flag: bool) {
        self.enableblend = flag;
    }
    pub fn blend_mode(
        &mut self,
        src_factor: wgpu::BlendFactor,
        dst_factor: wgpu::BlendFactor,
    ) {
        self.blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor,
                dst_factor,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }
    }

    pub fn draw(
        &mut self,
        texture: Option<Handle<TextureRes>>,
        sampler: Option<Handle<SamplerRes>>,
        vertices: &[f32],
        indices: Option<&[u16]>,
        vertices_len: u32,
        indices_len: u32,
        device: &RenderDevice,
        queue: &RenderQueue,
        target_format: wgpu::TextureFormat,
        asset_mgr_bindgroup: &Share<AssetMgr<RenderRes<BindGroup>>>,
        vb_allocator: &mut VertexBufferAllocator,
        pipelines: &mut SingleSpinePipelinePool,
        bind_group_layouts: &SingleSpineBindGroupLayout,
    ) {
        let shader = if let Some(shader) = &self.shader {
            shader
        } else {
            return;
        };
        let bind = if let Some(bind) = &self.bind {
            bind
        } else {
            return;
        };
        let (vb, bindgroup) = match shader {
            KeySpineShader::Colored => {
                let vb = if let Some(vb) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(vertices)) {
                    let mut result = VecMap::default();
                    result.insert(0, RenderVertices { slot: 0, buffer: EVerticesBufferUsage::EVBRange(Arc::new(vb)), buffer_range: Some(Range { start: 0, end: (vertices_len * 4) as u64  }), size_per_value: shader.vertices_bytes_per_element() as u64 });
                    result
                } else {
                    return;
                };
                let bindgroup = SpineBindGroup::colored(bind.0.clone(), device, asset_mgr_bindgroup, bind_group_layouts);
                (vb, bindgroup)
            },
            KeySpineShader::ColoredTextured => {
                let vb = if let Some(vb) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(vertices)) {
                    let mut result = VecMap::default();
                    result.insert(0, RenderVertices { slot: 0, buffer: EVerticesBufferUsage::EVBRange(Arc::new(vb)), buffer_range: Some(Range { start: 0, end: (vertices_len * 4) as u64  }), size_per_value: shader.vertices_bytes_per_element() as u64 });
                    result
                } else {
                    return;
                };
                let bindgroup = if let (Some(texture), Some(sampler)) = (texture, sampler) {
                    SpineBindGroup::colored_textured(bind.0.clone(), device, texture, sampler, asset_mgr_bindgroup, bind_group_layouts)
                } else {
                    return;
                };
                (vb, bindgroup)
            },
            KeySpineShader::TwoColoredTextured => {
                let vb = if let Some(vb) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(vertices)) {
                    let mut result = VecMap::default();
                    result.insert(0, RenderVertices { slot: 0, buffer: EVerticesBufferUsage::EVBRange(Arc::new(vb)), buffer_range: Some(Range { start: 0, end: (vertices_len * 4) as u64  }), size_per_value: shader.vertices_bytes_per_element() as u64 });
                    result
                } else {
                    return;
                };
                let bindgroup = if let (Some(texture), Some(sampler)) = (texture, sampler) {
                    SpineBindGroup::two_colored_textured(bind.0.clone(), device, texture, sampler, asset_mgr_bindgroup, bind_group_layouts)
                } else {
                    return;
                };
                (vb, bindgroup)
            },
        };
        
        let ib = if let Some(indices) = indices {
            if let Some(ib) = vb_allocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(indices)) {
                let temp = RenderIndices { buffer: EVerticesBufferUsage::EVBRange(Arc::new(ib)), buffer_range: Some(Range { start: 0, end: (indices_len * 2) as u64  }), format: wgpu::IndexFormat::Uint16 };
                Some(temp)
            } else {
                return;
            }
        } else {
            None
        };

        let blend = if self.enableblend {
            Some(self.blend.clone())
        } else {
            None
        };

        let key: KeySpinePipeline = KeySpinePipeline {
            key_shader: shader.clone(),
            key_state: KeyRenderPipelineState {
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: Some(wgpu::IndexFormat::Uint16),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    front_face: wgpu::FrontFace::Ccw,
                    unclipped_depth: true,
                    cull_mode: None,
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
                depth_stencil: None,
                target_state: vec![Some(wgpu::ColorTargetState { format: target_format, blend, write_mask: wgpu::ColorWrites::ALL })],
            },
        };
        let pipeline = pipelines.get(device, &key);

        let mut bindgroups = DrawBindGroups::default();
        bindgroups.insert_group(0, DrawBindGroup::Independ(bindgroup.bindgroup.clone()));
        self.bind_groups.push(bindgroup);

        let draw = DrawObj {
            pipeline,
            bindgroups,
            vertices: vb,
            vertex: Range { start: 0, end: vertices_len },
            instances: 0..1,
            indices: ib,
        };
        self.draws.list.push(Arc::new(draw));
    }
}