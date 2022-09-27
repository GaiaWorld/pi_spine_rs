use pi_hash::XHashMap;
use render_pipeline_key::{fragment_state::gen_fragment_state_key, pipeline_key::{PipelineKey, PipelineKeyCalcolator, gen_pipeline_key}};

use crate::shaders::{SpineShader, SpineShaderPool, EShader};



pub type SpinePipelineKey = PipelineKey;

pub trait SpinePipelinePool {
    fn record_spine_pipeline_colored(&mut self, key: SpinePipelineKey, pipeline: SpinePipeline);
    fn record_spine_pipeline_colored_textured(&mut self, key: SpinePipelineKey, pipeline: SpinePipeline);
    fn record_spine_pipeline_colored_textured_two(&mut self, key: SpinePipelineKey, pipeline: SpinePipeline);
    fn get_spine_pipeline_colored(&self, key: SpinePipelineKey) -> Option<&SpinePipeline>;
    fn get_spine_pipeline_colored_textured(&self, key: SpinePipelineKey) -> Option<&SpinePipeline>;
    fn get_spine_pipeline_colored_textured_two(&self, key: SpinePipelineKey) -> Option<&SpinePipeline>;
}

pub struct SpinePipelinePoolSimple {
    colored: XHashMap<PipelineKey, SpinePipeline>,
    colored_textured: XHashMap<PipelineKey, SpinePipeline>,
    colored_textured_two: XHashMap<PipelineKey, SpinePipeline>,
}

impl Default for SpinePipelinePoolSimple {
    fn default() -> Self {
        Self { colored: XHashMap::default(), colored_textured: XHashMap::default(), colored_textured_two: XHashMap::default() }
    }
}

impl SpinePipelinePool for SpinePipelinePoolSimple {
    fn record_spine_pipeline_colored(&mut self, key: SpinePipelineKey, pipeline: SpinePipeline) {
        self.colored.insert(key, pipeline);
    }

    fn record_spine_pipeline_colored_textured(&mut self, key: SpinePipelineKey, pipeline: SpinePipeline) {
        self.colored_textured.insert(key, pipeline);
    }

    fn record_spine_pipeline_colored_textured_two(&mut self, key: SpinePipelineKey, pipeline: SpinePipeline) {
        self.colored_textured_two.insert(key, pipeline);
    }

    fn get_spine_pipeline_colored(&self, key: SpinePipelineKey) -> Option<&SpinePipeline> {
        self.colored.get(&key)
    }

    fn get_spine_pipeline_colored_textured(&self, key: SpinePipelineKey) -> Option<&SpinePipeline> {
        self.colored_textured.get(&key)
    }

    fn get_spine_pipeline_colored_textured_two(&self, key: SpinePipelineKey) -> Option<&SpinePipeline> {
        self.colored_textured_two.get(&key)
    }
}

pub struct SpinePipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
}

impl SpinePipeline {
    pub fn check<'a, P: SpinePipelinePool, P0: SpineShaderPool>(
        shader: EShader,
        device: &wgpu::Device,
        shader_pool: &'a P0,
        pipelines: &'a mut P,
        targets: &[wgpu::ColorTargetState],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> PipelineKey {
        match shader {
            EShader::Colored => Self::check_colored(device, shader_pool, pipelines, targets, primitive, depth_stencil),
            EShader::ColoredTextured => Self::check_colored_textured(device, shader_pool, pipelines, targets, primitive, depth_stencil),
            EShader::TwoColoredTextured => Self::check_colored_textured_two(device, shader_pool, pipelines, targets, primitive, depth_stencil),
        }
    }
    pub fn get<'a, P: SpinePipelinePool>(
        shader: EShader,
        pipelines: &'a P,
        key: PipelineKey,
    ) -> Option<&'a Self> {
        match shader {
            EShader::Colored => pipelines.get_spine_pipeline_colored(key),
            EShader::ColoredTextured => pipelines.get_spine_pipeline_colored_textured(key),
            EShader::TwoColoredTextured => pipelines.get_spine_pipeline_colored_textured_two(key),
        }
    }
    pub fn check_colored<'a, P: SpinePipelinePool, P0: SpineShaderPool>(
        device: &wgpu::Device,
        shader_pool: &'a P0,
        pipelines: &'a mut P,
        targets: &[wgpu::ColorTargetState],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> PipelineKey {
        let mut calcolator = PipelineKeyCalcolator::new();

        gen_pipeline_key(&mut calcolator, &primitive, &depth_stencil, 0, 1);
        gen_fragment_state_key(&mut calcolator, &targets[0]);

        let key = calcolator.key;

        match pipelines.get_spine_pipeline_colored(key) {
            Some(_) => {},
            None => {
                let shader = shader_pool.get_spine_shader_colored();
                let pipeline = Self::create(device, shader, targets, primitive, depth_stencil, Some("Colored"));
                pipelines.record_spine_pipeline_colored(key, pipeline);
            },
        }

        key
    }

    pub fn check_colored_textured<'a, P: SpinePipelinePool, P0: SpineShaderPool>(
        device: &wgpu::Device,
        shader_pool: &'a P0,
        pipelines: &'a mut P,
        targets: &[wgpu::ColorTargetState],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> PipelineKey {
        let mut calcolator = PipelineKeyCalcolator::new();

        gen_pipeline_key(&mut calcolator, &primitive, &depth_stencil, 0, 1);
        gen_fragment_state_key(&mut calcolator, &targets[0]);

        let key = calcolator.key;

        match pipelines.get_spine_pipeline_colored_textured(key) {
            Some(_) => {},
            None => {
                let shader = shader_pool.get_spine_shader_colored_textured();
                let pipeline = Self::create(device, shader, targets, primitive, depth_stencil, Some("ColoredTextured"));
                pipelines.record_spine_pipeline_colored_textured(key, pipeline);
            },
        }

        key
    }

    pub fn check_colored_textured_two<'a, P: SpinePipelinePool, P0: SpineShaderPool>(
        device: &wgpu::Device,
        shader_pool: &'a P0,
        pipelines: &'a mut P,
        targets: &[wgpu::ColorTargetState],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> PipelineKey {
        let mut calcolator = PipelineKeyCalcolator::new();

        gen_pipeline_key(&mut calcolator, &primitive, &depth_stencil, 0, 1);
        gen_fragment_state_key(&mut calcolator, &targets[0]);

        let key = calcolator.key;

        match pipelines.get_spine_pipeline_colored_textured_two(key) {
            Some(_) => {},
            None => {
                let shader = shader_pool.get_spine_shader_colored_textured_two();
                let pipeline = Self::create(device, shader, targets, primitive, depth_stencil, Some("ColoredTexturedTwo"));
                pipelines.record_spine_pipeline_colored_textured_two(key, pipeline);
            },
        }

        key
    }

    fn create(
        device: &wgpu::Device,
        shader: & SpineShader,
        targets: &[wgpu::ColorTargetState],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
        label: Option<&str>,
    ) -> Self {
        let vertex_layouts = vec![
            wgpu::VertexBufferLayout {
                array_stride: shader.attributes_bytes as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &shader.attributes,
            },
        ];

        let vs_state = wgpu::VertexState {
            module: &shader.vs_module,
            entry_point: "main",
            buffers: &vertex_layouts,
        };
        let fs_state = wgpu::FragmentState {
            module: &shader.fs_module,
            entry_point: "main",
            targets,
        };
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: shader.bind_group_layouts().as_slice(),
                push_constant_ranges: &[],
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label,
                layout: Some(&pipeline_layout),
                vertex: vs_state,
                fragment: Some(fs_state),
                primitive,
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                multiview: None,
            }
        );

        Self {
            pipeline,
            pipeline_layout,
        }
    }
}