use std::{ops::Range, sync::Arc};

use bevy::prelude::{Resource};
use pi_assets::{asset::{Handle, GarbageEmpty}, mgr::AssetMgr};
use pi_hash::XHashMap;


use pi_map::{vecmap::VecMap, smallvecmap::SmallVecMap};
use pi_render::{
    renderer::{
        draw_obj::{DrawObj, DrawBindGroups, DrawBindGroup},
        sampler::SamplerRes,
        pipeline::KeyRenderPipelineState,
        vertices::{RenderVertices, EVerticesBufferUsage, RenderIndices},
        draw_obj_list::DrawList, vertex_buffer::VertexBufferAllocator,
        texture::*
    },
    rhi::{
        asset::{TextureRes, RenderRes},
        device::RenderDevice, RenderQueue, bind_group::BindGroup, PrimitiveState,
        sampler::SamplerDesc, options::RenderOptions
    }, asset::TAssetKeyU64
};
use pi_share::Share;

use crate::{shaders::{KeySpineShader, KeySpinePipeline, SingleSpinePipelinePool, SingleSpineBindGroupLayout}, binds::param::{BindBufferAllocator, SpineBindBufferUsage}, bind_groups::SpineBindGroup, FORMAT};


#[derive(Resource)]
pub struct SpineResource {
    pipelines: SingleSpinePipelinePool,
    bind_group_layouts: SingleSpineBindGroupLayout,
    vballocator: VertexBufferAllocator,
    bindallocator: BindBufferAllocator,
    asset_mgr_bindgroup: Share<AssetMgr<RenderRes<BindGroup>>>,
}
impl SpineResource {
    pub fn new(device: &RenderDevice) -> Self {
        Self {
            pipelines: SingleSpinePipelinePool::new(device),
            bind_group_layouts: SingleSpineBindGroupLayout::new(device),
            vballocator: VertexBufferAllocator::new(),
            bindallocator: BindBufferAllocator::new(),
            asset_mgr_bindgroup: AssetMgr::<RenderRes::<BindGroup>>::new(GarbageEmpty(), false, 1024, 30 * 1000),
        }
    }
}

pub struct SpineDraw {
    bind_key: usize,
    vertices: Vec<f32>,
    indices: Option<Vec<u16>>,
    verticeslen: u32,
    indiceslen: u32,
    texture: Option<Handle<TextureRes>>,
    sampler: Option<Handle<SamplerRes>>,
    shader: KeySpineShader,
    pipeline: KeySpinePipeline,
}

pub struct RendererAsync {
    pub(crate) binds: Vec<SpineBindBufferUsage>,
    pub(crate) bind_groups: Vec<SpineBindGroup>,
    pub(crate) draws: Vec<SpineDraw>,
    pub(crate) drawobjs: DrawList,
    pub(crate) shader: Option<KeySpineShader>,
    pub(crate) blend: wgpu::BlendState,
    pub(crate) enableblend: bool,
    pub(crate) textures: XHashMap<u64, Handle<TextureRes>>,
    pub(crate) samplers: XHashMap<SamplerDesc, Handle<SamplerRes>>,
    uniform_param: Vec<Vec<f32>>,
    texture: Option<Handle<TextureRes>>,
    sampler: Option<Handle<SamplerRes>>,
    pub target_format: wgpu::TextureFormat,
}
impl RendererAsync {
    pub fn new() -> Self {
        Self {
            binds: vec![],
            bind_groups: vec![],
            draws: vec![],
            drawobjs: DrawList::default(),
            shader: None,
            blend: wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            enableblend: true,
            uniform_param: vec![],
            texture: None,
            sampler: None,
            textures: XHashMap::default(),
            samplers: XHashMap::default(),
            target_format: wgpu::TextureFormat::Bgra8Unorm,
        }
    }
    pub fn drawlist(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        resource: &mut SpineResource,
        asset_samplers: &Share<AssetMgr<SamplerRes>>,
        asset_textures: &Share<AssetMgr<TextureRes>>,
    ) -> &DrawList {
        let mut binds = vec![];
        self.uniform_param.drain(..).for_each(|uniform_param| {
            let bind = resource.bindallocator.allocate(device, queue, bytemuck::cast_slice(&uniform_param));
            if let Some(bind) = &bind {
                binds.push(bind.clone());
            }
        });

        self.draws.drain(..).for_each(|draw| {
            let bind = if let Some(bind) = binds.get(draw.bind_key) {
                bind
            } else {
                // log::warn!("drawlist Err: Bind");
                return;
            };
            let (vb, bindgroup) = match &draw.shader {
                KeySpineShader::Colored => {
                    let vb = if let Some(vb) = resource.vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&draw.vertices)) {

                        let mut result = SmallVecMap::default();
                        result.insert(0, RenderVertices { slot: 0, buffer: EVerticesBufferUsage::EVBRange(Arc::new(vb)), buffer_range: Some(Range { start: 0, end: (draw.verticeslen * 4) as u64  }), size_per_value: draw.shader.vertices_bytes_per_element() as u64 });
                        result
                    } else {
                        // log::warn!("drawlist Err: vb");
                        return;
                    };

                    let bindgroup = SpineBindGroup::colored(bind.0.clone(), device, &resource.asset_mgr_bindgroup, &resource.bind_group_layouts);
                    (vb, bindgroup)
                },
                KeySpineShader::ColoredTextured => {
                    let vb = if let Some(vb) = resource.vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&draw.vertices)) {

                        let mut result = SmallVecMap::default();
                        result.insert(0, RenderVertices { slot: 0, buffer: EVerticesBufferUsage::EVBRange(Arc::new(vb)), buffer_range: Some(Range { start: 0, end: (draw.verticeslen * 4) as u64  }), size_per_value: draw.shader.vertices_bytes_per_element() as u64 });
                        result
                    } else {
                        // log::warn!("drawlist Err: vb");
                        return;
                    };
                    
                    match (draw.texture.clone(), draw.sampler.clone()) {
                        (Some(texture), Some(sampler)) => {
                            let bindgroup = SpineBindGroup::two_colored_textured(bind.0.clone(), device, texture, sampler, &resource.asset_mgr_bindgroup, &resource.bind_group_layouts);
                            (vb, bindgroup)
                        },
                        _ => {
                            // log::warn!("drawlist Err: tex");
                            return;
                        },
                    }
                },
                KeySpineShader::TwoColoredTextured => {
                    
                    // log::warn!("VB : ");
                    let vb = if let Some(vb) = resource.vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&draw.vertices)) {

                        let mut result = SmallVecMap::default();
                        result.insert(0, RenderVertices { slot: 0, buffer: EVerticesBufferUsage::EVBRange(Arc::new(vb)), buffer_range: Some(Range { start: 0, end: (draw.verticeslen * 4) as u64  }), size_per_value: draw.shader.vertices_bytes_per_element() as u64 });
                        result
                    } else {
                        // log::warn!("drawlist Err: vb");
                        return;
                    };
                    match (draw.texture.clone(), draw.sampler.clone()) {
                        (Some(texture), Some(sampler)) => {
                            let bindgroup = SpineBindGroup::two_colored_textured(bind.0.clone(), device, texture, sampler, &resource.asset_mgr_bindgroup, &resource.bind_group_layouts);
                            (vb, bindgroup)
                        },
                        // _ => {
                        //     log::warn!("drawlist Err: tex {:?}, {:?}", );
                        //     return
                        // },
                        (None, None) => {
                            // log::warn!("drawlist Err: tex [None, None]");
                            return;
                        },
                        (None, Some(_)) => {
                            // log::warn!("drawlist Err: tex [None, Sampler]");
                            return;
                        },
                        (Some(_), None) => {
                            // log::warn!("drawlist Err: tex [Tex, None]");
                            return;
                        },
                    }
                },
            };
            
            // log::warn!("IB : ");

            let ib = if let Some(indices) = &draw.indices {
                if let Some(ib) = resource.vballocator.create_not_updatable_buffer_for_index(device, queue, bytemuck::cast_slice(indices)) {
                    let temp = RenderIndices { buffer: EVerticesBufferUsage::EVBRange(Arc::new(ib)), buffer_range: Some(Range { start: 0, end: (draw.indiceslen * 2) as u64  }), format: wgpu::IndexFormat::Uint16 };
                    Some(temp)
                } else {
                    // log::warn!("drawlist Err: ib");
                    return;
                }
            } else {
                None
            };
            
            // log::warn!("Pipeline : ");

            let pipeline = resource.pipelines.get(device, &draw.pipeline);
            let mut bindgroups = DrawBindGroups::default();
            bindgroups.insert_group(0, DrawBindGroup::Independ(bindgroup.bindgroup.clone()));
            self.bind_groups.push(bindgroup);

            let draw = DrawObj {
                pipeline,
                bindgroups,
                vertices: vb,
                vertex: Range { start: 0, end: draw.verticeslen },
                instances: 0..1,
                indices: ib,
            };

            self.drawobjs.list.push(Arc::new(draw));
            // log::warn!("drawlist : {:?}", self.drawobjs.list.len());
        });

        &self.drawobjs
    }
    pub fn reset(&mut self) {
        self.bind_groups.clear();
        self.binds.clear();
        self.draws.clear();
        self.drawobjs.list.clear();
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
        uniform_param: Vec<f32>,
    ) {
        self.uniform_param.push(uniform_param);
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

    pub fn texture(
        &mut self,
        texture: Option<Handle<TextureRes>>,
        sampler: Option<Handle<SamplerRes>>,
    ) {
        // log::warn!("texture {:?}", texture);
        self.texture = texture;
        self.sampler = sampler;
    }

    pub fn record_texture(
        &mut self,
        key_texture: u64,
        texture: Handle<TextureRes>,
    ) {
        // log::warn!("record_texture {:?}", key_texture);
        self.textures.insert(key_texture, texture);
    }

    pub fn record_sampler(
        &mut self,
        key_sampler: SamplerDesc,
        sampler: Handle<SamplerRes>,
    ) {
        // log::warn!("record_sampler {:?}", key_sampler);
        self.samplers.insert(key_sampler, sampler);
    }

    pub fn remove_texture(
        &mut self,
        key_texture: u64,
    ) {
        self.textures.remove(&key_texture);
    }

    pub fn draw(
        &mut self,
        vertices: Vec<f32>,
        indices: Option<Vec<u16>>,
        vertices_len: u32,
        indices_len: u32,
        renderopt: &RenderOptions,
    ) {
        let shader = if let Some(shader) = &self.shader {
            shader
        } else {
            // log::warn!("draw Err: shader");
            return;
        };

        let indices = if let Some(indices) = indices {
            Some(indices)
        } else {
            None
        };

        if self.uniform_param.len() == 0 {
            // log::warn!("draw Err: uniform_param");
            return;
        }

        match shader {
            KeySpineShader::Colored => {
            },
            KeySpineShader::ColoredTextured => {
                if self.texture.is_none() && self.sampler.is_none() {
                    return;
                }
            },
            KeySpineShader::TwoColoredTextured => {
                if self.texture.is_none() && self.sampler.is_none() {
                    return;
                }
            },
        };

        let blend = if self.enableblend {
            Some(self.blend.clone())
        } else {
            None
        };

        // let unclipped_depth = renderopt.features & wgpu::Features::DEPTH_CLIP_CONTROL == wgpu::Features::DEPTH_CLIP_CONTROL;

        let key: KeySpinePipeline = KeySpinePipeline {
            key_shader: shader.clone(),
            key_state: KeyRenderPipelineState {
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: Some(wgpu::IndexFormat::Uint16),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    front_face: wgpu::FrontFace::Ccw,
                    unclipped_depth: false,
                    cull_mode: None,
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
                depth_stencil: None,
                target_state: vec![Some(wgpu::ColorTargetState { format: self.target_format, blend, write_mask: wgpu::ColorWrites::ALL })],
            },
        };
        
        let draw = SpineDraw {
            bind_key: self.uniform_param.len() - 1,
            vertices,
            indices,
            verticeslen: vertices_len,
            indiceslen: indices_len,
            texture: self.texture.clone(),
            sampler: self.sampler.clone(),
            shader: shader.clone(),
            pipeline: key,
        };

        self.draws.push(draw);
        // log::warn!("Draws: {:?}", self.draws.len());
    }
}
