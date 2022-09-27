use render_data_container::{TextureID, TexturePool, TGeometryBufferID, GeometryBufferPool};
use render_data_container::{Number, Matrix};
use render_pipeline_key::pipeline_key::PipelineKey;

use crate::{mesh::{Mesh, VertexAttribute}, vec_set, shaders::{EShader, SpineShaderPool}, MAX_VERTICES, pipeline::{SpinePipelinePool, SpinePipeline}};


pub struct PolygonBatcher<GBID: TGeometryBufferID, TID: TextureID> {
    pub src_factor: wgpu::BlendFactor,
    pub dst_factor: wgpu::BlendFactor,
    pub blend: Option<wgpu::BlendState>,
    pub shader: EShader,
    pub meshes: Vec<Mesh<GBID, TID>>,
    pub is_drawing: bool,
    pub draw_calls: usize,
    pub vertices_length: usize,
    pub indices_length: usize,
    pub last_texture_key: Option<TID>,
    pub attributes: Vec<VertexAttribute>,
    pub mask_flag: (Number, Number, Number, Number),
    pub color: (Number, Number, Number, Number),
    pub mvp_matrix: Matrix,
    pub elements_per_vertex: u32,
    pub pipeline_key: Option<PipelineKey>,
}

impl<GBID: TGeometryBufferID, TID: TextureID> PolygonBatcher<GBID, TID> {
    pub fn new(
        two_color_tint: bool, 
    ) -> Self {
        let attributes = if two_color_tint {
            vec![
                VertexAttribute::position_2(),
                VertexAttribute::color(),
                VertexAttribute::texcoords(),
                VertexAttribute::color2(),
            ]
        } else {
            vec![
                VertexAttribute::position_2(),
                VertexAttribute::color(),
                VertexAttribute::texcoords(),
            ]
        };
        let elements_per_vertex = VertexAttribute::elements(&attributes);
        let shader = if two_color_tint { EShader::ColoredTextured } else { EShader::TwoColoredTextured };
        Self {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::Zero,
            blend: None,
            meshes: vec![],
            is_drawing: false,
            draw_calls: 0,
            vertices_length: 0,
            indices_length: 0,
            last_texture_key: None,
            attributes,
            mask_flag: (0., 0., 0., 0.),
            color: (1., 1., 1., 1.),
            mvp_matrix: Matrix::identity(),
            elements_per_vertex,
            shader: shader,
            pipeline_key: None,
        }
    }
    pub fn color_mut(&mut self, r: Number, g: Number, b: Number, a: Number) {
        self.color = (r, g, b, a);
    }
    pub fn begin<'a, SP: SpineShaderPool, SPP: SpinePipelinePool>(
        &mut self,
        device: & wgpu::Device,
        renderpass: &mut wgpu::RenderPass<'a>,
        target_format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
        shaders: &mut SP,
        pipelines: &mut SPP,
    ) {
        self.draw_calls = 0;
        self.is_drawing = true;

        let color_target = wgpu::ColorTargetState {
            format: target_format,
            blend: self.blend,
            write_mask: wgpu::ColorWrites::ALL,
        };

        let pipeline_key = SpinePipeline::check(self.shader, device, shaders, pipelines, &[color_target], wgpu::PrimitiveState::default(), depth_stencil);
        self.pipeline_key = Some(pipeline_key);
    }
    pub fn set_blend_mode(&mut self, src_factor: wgpu::BlendFactor, dst_factor: wgpu::BlendFactor, ) {
        self.src_factor = src_factor;
        self.dst_factor = dst_factor;

        let blend = Some(
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor,
                    dst_factor,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            }
        );
        self.blend = blend;
    }
    pub fn draw<'a, SP: SpineShaderPool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &'a mut self,
        texture_key: Option<TID>,
        texture_pool: &wgpu::TextureView,
        vertices: &[f32],
        indices: &[u16],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderpass: &mut wgpu::RenderPass<'a>,
        shaders: &mut SP,
        textures: &TP,
        geo_buffers: &'a mut GBP,
    ) {
        let temp_vertices_length = vertices.len();
        let temp_indices_length = indices.len();

        let last_texture_key = self.last_texture_key;
        let vertices_length = self.vertices_length;
        let indices_length = self.indices_length;


        if texture_key != last_texture_key {
            self.last_texture_key = texture_key;
            self.flush(device, queue, renderpass, shaders, textures, geo_buffers);
        } else {
            if vertices_length + temp_vertices_length > MAX_VERTICES as usize * self.elements_per_vertex as usize
                || indices_length + temp_indices_length > MAX_VERTICES as usize * 3
            {
                self.flush(device, queue, renderpass, shaders, textures, geo_buffers);
            } else {
                let mesh = self.meshes.get_mut(self.draw_calls).unwrap();
                vec_set(mesh.get_vertices_mut(), vertices, 0);
            }
        }
    }
    fn flush<'a, SP: SpineShaderPool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &'a mut self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        renderpass: &mut wgpu::RenderPass<'a>, 
        shaders: &SP, 
        textures: &TP,
        geo_buffers: &'a mut GBP,
    ) {
        if self.vertices_length == 0 {
            // self.meshes.get_mut(self.draw_calls).unwrap()
        } else {
    
            self.draw_calls = self.draw_calls + 1;
            self.vertices_length = 0;
            self.indices_length = 0;

            if self.meshes.len() <= self.draw_calls {
                let mut mesh = Mesh::new();
                mesh.init(device, self.shader, shaders, geo_buffers);
                self.meshes.push(mesh);
            }

            let mesh = self.meshes.get_mut(self.draw_calls - 1).unwrap();
            mesh.draw(device, queue, renderpass, shaders, textures, geo_buffers);
        }
    }
    pub fn end<'a, SP: SpineShaderPool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderpass: &mut wgpu::RenderPass<'a>,
        shaders: &SP,
        textures: &TP,
        geo_buffers: &'a mut GBP,
    ) {
        let shader = self.shader;
        let last_texture_key = self.last_texture_key;
        self.last_texture_key = None;

        if !self.is_drawing {

        } else {
            self.is_drawing = false;
            if self.vertices_length > 0 || self.indices_length > 0 {
                self.vertices_length = 0;
                self.indices_length = 0;
                let mesh = self.meshes.get_mut(self.draw_calls).unwrap();
                mesh.draw(device, queue, renderpass, shaders, textures, geo_buffers);
            }
        }
    }
}