use render_data_container::{TextureID, TexturePool, TGeometryBufferID, GeometryBufferPool};
use render_material::{material::{UniformKindFloat4, UniformKindMat4}};
use render_data_container::{Number};
use render_pipeline_key::pipeline_key::PipelineKey;
use wgpu::DepthStencilState;

use crate::{mesh::Mesh, shaders::{EShader, SpineShaderPool}, pipeline::{SpinePipelinePool, SpinePipeline}};


pub struct  MeshRenderer<GBID: TGeometryBufferID, TID: TextureID> {
    mesh: Mesh<GBID, TID>,
    blend: Option<wgpu::BlendState>,
    shader: EShader,
    pipeline_key: Option<PipelineKey>,
}

impl<GBID: TGeometryBufferID, TID: TextureID> MeshRenderer<GBID, TID> {
    pub fn new() -> Self {
        Self {
            mesh: Mesh::new(),
            blend: None,
            shader: EShader::Colored,
            pipeline_key: None,
        }
    }
    pub fn update<SP: SpineShaderPool, SPP: SpinePipelinePool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Number],
        indices: &[u16],
        shader: EShader,
        mvp: UniformKindMat4,
        mask_flag: UniformKindFloat4,
        src_factor: wgpu::BlendFactor,
        dst_factor: wgpu::BlendFactor,
        target_format: wgpu::TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        texture_key: Option<TID>,
        shaders: &mut SP,
        pipelines: &mut SPP,
        textures: &TP,
        geo_buffers: &mut GBP,
    ) {
        self.mesh.init(device, shader, shaders, geo_buffers);
        self.mesh.mvp_matrix(queue, mvp);
        self.mesh.mask_flag(queue, mask_flag);
        self.mesh.texture(device, texture_key, shaders, textures);
        self.mesh.set_vertices(device, queue, vertices, geo_buffers);
        self.mesh.set_indices(device, queue, indices, geo_buffers);
        self.shader = shader;
        self.blend = Some(
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor,
                    dst_factor,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            }
        );
        let targets = [
            wgpu::ColorTargetState {
                format: target_format,
                blend: self.blend,
                write_mask: wgpu::ColorWrites::ALL,
            }
        ];
        self.pipeline_key = Some(SpinePipeline::check(shader, device, shaders, pipelines, &targets, wgpu::PrimitiveState::default(), depth_stencil));
    }
    pub fn draw<'a, SPP: SpinePipelinePool, SP: SpineShaderPool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &'a self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderpass: &mut wgpu::RenderPass<'a>,
        pipelines: &'a SPP,
        shaders: &'a SP,
        textures: &'a TP,
        geo_buffers: &'a GBP,
    ) {
        match self.pipeline_key {
            Some(key) => {
                // println!(">>>>>>>>>>>>>>>> {}", key);
                match SpinePipeline::get(self.shader, pipelines, key) {
                    Some(pipeline) => {
                        renderpass.set_pipeline(&pipeline.pipeline);
                        self.mesh.draw(device, queue, renderpass, shaders, textures, geo_buffers);
                    },
                    None => {},
                }
            },
            None => {
                
            },
        }
    }
    pub fn update_uniform<'a, SP: SpineShaderPool, TP: TexturePool<TID>>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shaders: &'a SP,
        textures: &'a TP,
    ) {
        self.mesh.update_uniform(device, queue, shaders, textures);
    }
}

pub struct MeshRendererPool<GBID: TGeometryBufferID, TID: TextureID> {
    renderers: Vec<MeshRenderer<GBID, TID>>,
    counter: usize,
}

impl<GBID: TGeometryBufferID, TID: TextureID> Default for  MeshRendererPool<GBID, TID> {
    fn default() -> Self {
        Self { renderers: vec![], counter: 0 }
    }
}

impl<GBID: TGeometryBufferID, TID: TextureID>  MeshRendererPool<GBID, TID> {
    pub fn insert<SP: SpineShaderPool, SPP: SpinePipelinePool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Number],
        indices: &[u16],
        shader: EShader,
        mvp: UniformKindMat4,
        mask_flag: UniformKindFloat4,
        src_factor: wgpu::BlendFactor,
        dst_factor: wgpu::BlendFactor,
        target_format: wgpu::TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        texture_key: Option<TID>,
        shaders: &mut SP,
        pipelines: &mut SPP,
        textures: &TP,
        geo_buffers: &mut GBP,
    ) {
        self.counter += 1;
        if self.renderers.len() < self.counter {
            let renderer = MeshRenderer::new();
            self.renderers.push(renderer);
        }

        self.renderers.get_mut(self.counter - 1).unwrap().update(device, queue, vertices, indices, shader, mvp, mask_flag, src_factor, dst_factor, target_format, depth_stencil, texture_key, shaders, pipelines, textures, geo_buffers);
    }
    pub fn reset(&mut self) {
        self.counter = 0;
    }
    pub fn draw<'a, SP: SpineShaderPool, SPP: SpinePipelinePool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &'a self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderpass: &mut wgpu::RenderPass<'a>,
        pipelines: &'a SPP,
        shaders: &'a SP,
        textures: &'a TP,
        geo_buffers: &'a GBP,
    ) {
        for i in 0.. self.counter {
            let renderer = self.renderers.get(i).unwrap();
            renderer.draw(device, queue, renderpass, pipelines, shaders, textures, geo_buffers);
        }
    }
    
    pub fn update_uniforms<'a, SP: SpineShaderPool, TP: TexturePool<TID>>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shaders: &'a SP,
        textures: &'a TP,
    ) {
        for i in 0.. self.counter {
            match self.renderers.get_mut(i) {
                Some(renderer) => renderer.update_uniform(device, queue, shaders, textures),
                None => {},
            };
        }
    }
}