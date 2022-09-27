use render_data_container::{TextureID, TVertexBufferKindKey, TexturePool, TGeometryBufferID, GeometryBufferPool, GeometryBuffer, EVertexDataFormat};
use render_geometry::{geometry::Geometry};
use render_material::{material::{Material, UniformKindFloat4, UniformKindMat4}};
use wgpu::util::DeviceExt;

use crate::{MAX_VERTICES, error::ESpineError, vec_set, pipeline::SpinePipelinePool, material::{TSpineMaterialUpdate, SpineMaterialColored, SpineMaterialBlockKindKey, SpineVertexBufferKindKey, SpineMaterialColoredTextured, SpineMaterialColoredTexturedTwo}, shaders::{EShader, SpineShaderPool}};


pub struct Mesh<GBID: TGeometryBufferID, TID: TextureID> {
    shader: Option<EShader>,
    material: Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
    geometry: Geometry<SpineVertexBufferKindKey, GBID>,
    vertices: Vec<f32>,
    indices: Vec<u16>,
    attributes: Vec<VertexAttribute>,
    num_vertices: u32,
    num_indices: u32,
    dirty_vertices: bool,
    dirty_indices: bool,
    vertices_length: u32,
    indices_length: u32,
    element_per_vertex: u32,
    vertices_buffer_id: Option<GBID>,
    indices_buffer_id: Option<GBID>,
    max_vertices: usize,
    max_indices: usize,
}

impl<GBID: TGeometryBufferID, TID: TextureID> Mesh<GBID, TID> {
    pub fn new() -> Self {
        Self {
            shader: None,
            material: Material::default(),
            geometry: Geometry::default(),
            vertices: vec![],
            indices: vec![],
            attributes: vec![],
            num_vertices: 0,
            num_indices: 0,
            dirty_vertices: false,
            dirty_indices: false,
            vertices_length: 0,
            indices_length: 0,
            element_per_vertex: 0,
            vertices_buffer_id: None,
            indices_buffer_id: None,
            max_vertices: 0,
            max_indices: 0,
        }
    }
    pub fn init<GBP: GeometryBufferPool<GBID>, SP: SpineShaderPool>(
        &mut self, 
        device: &wgpu::Device, 
        shader: EShader, 
        shader_pool: &SP,
        geo_buffers: &mut GBP,
    ) {
        match self.shader {
            Some(v) => {
                if v != shader {
                    Self::init_material(&mut self.material, device, shader, shader_pool);
                }
            },
            None => {
                Self::init_material(&mut self.material, device, shader, shader_pool);
            },
        }

        self.attributes = match shader {
            EShader::Colored => {
                vec![
                    VertexAttribute::position_2(),
                    VertexAttribute::color(),
                ]
            },
            EShader::ColoredTextured => {
                vec![
                    VertexAttribute::position_2(),
                    VertexAttribute::color(),
                    VertexAttribute::texcoords(),
                ]
            },
            EShader::TwoColoredTextured => {
                vec![
                    VertexAttribute::position_2(),
                    VertexAttribute::color(),
                    VertexAttribute::texcoords(),
                    VertexAttribute::color2(),
                ]
            },
        };

        let element_per_vertex = VertexAttribute::elements(&self.attributes);
        if element_per_vertex > self.element_per_vertex {
            self.num_vertices = MAX_VERTICES as u32 * element_per_vertex;
            self.num_indices = MAX_VERTICES as u32 * 3;
            self.vertices = vec![];
            self.indices = vec![];
            for _ in 0..self.num_vertices {
                self.vertices.push(0.);
            }
            for _ in 0..self.num_indices {
                self.indices.push(0);
            }
            // println!(">>>>>>>>>>>>>>>> 00 {}", self.indices.len());
            self.max_vertices = MAX_VERTICES as usize;
            self.max_indices = MAX_VERTICES as usize * 3;
        }

        self.vertices_buffer_id = match self.vertices_buffer_id {
            Some(id) => Some(id),
            None => Some(geo_buffers.insert(GeometryBuffer::new(true, EVertexDataFormat::F32, false))),
        };
        self.indices_buffer_id = match self.indices_buffer_id {
            Some(id) => Some(id),
            None => Some(geo_buffers.insert(GeometryBuffer::new(true, EVertexDataFormat::U16, true))),
        };

        self.geometry.reset();
        self.geometry.set_indices(self.indices_buffer_id);
        match shader {
            EShader::Colored => {
                self.geometry.set(EShader::VERTEX_COLORED, self.vertices_buffer_id);
            },
            EShader::ColoredTextured => {
                self.geometry.set(EShader::VERTEX_COLORED_TEXTURED, self.vertices_buffer_id);
            },
            EShader::TwoColoredTextured => {
                self.geometry.set(EShader::VERTEX_COLORED_TEXTURED_TWO, self.vertices_buffer_id);
            },
        };
        self.element_per_vertex = element_per_vertex;
        self.shader = Some(shader);
        self.dirty_vertices = false;
        self.dirty_indices = false;
        self.vertices_length = 0;
        self.indices_length = 0;
    }
    pub fn element_per_vertex(&self) -> u32 {
        VertexAttribute::elements(&self.attributes)
    }
    pub fn get_attributes(&self) -> &Vec<VertexAttribute> {
        &self.attributes
    }

    pub fn max_vertices(&self) -> u32 {
        self.max_vertices as u32
    }
    pub fn num_vertices(&self) -> u32 {
        self.vertices_length / self.element_per_vertex()
    }
    pub fn set_vertices_length(&mut self, length: u32) {
        self.dirty_vertices = true;
        self.vertices_length = length;
    }
    pub fn get_vertices(& self) -> & Vec<f32>  {
        & self.vertices
    }
    pub fn get_vertices_mut(&mut self) -> &mut Vec<f32>  {
        &mut self.vertices
    }

    pub fn max_indices(&self) -> u32 {
        self.max_indices as u32
    }
    pub fn num_indices(&self) -> u32 {
        self.indices_length
    }
    pub fn set_indices_length(&mut self, length: u32) {
        self.dirty_indices = true;
        self.indices_length = length;
    }
    pub fn get_indices_mut(&mut self) -> &mut Vec<u16> {
        &mut self.indices
    }
    pub fn get_indices(& self) -> & Vec<u16> {
        & self.indices
    }

    pub fn get_vertex_size_in_floats(&self) -> u32 {
        let mut size = 0;
        for v in self.attributes.iter() {
            size += v.num_elements;
        }

        size
    }

    pub fn set_vertices<GBP: GeometryBufferPool<GBID>>(
        &mut self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        vertices: &[f32],
        geo_buffers: &mut GBP,
    ) -> Result<(), ESpineError> {
        self.dirty_vertices = true;
        // if vertices.len() > self.vertices.len() {
        //     // println!(">>>>>>>>>>>>>>>> V0");
        //     Err(ESpineError::MeshCanntStoreMoreThanMaxVertices)
        // } else {
        //     // println!(">>>>>>>>>>>>>>>> V1");
        //     vec_set(&mut self.vertices, vertices, 0);
        //     self.vertices_length = vertices.len() as u32;
        //     if self.vertices_buffer.is_none() {
        //         self.vertices_buffer = Some(device.create_buffer_init(
        //             &wgpu::util::BufferInitDescriptor {
        //                 label: None,
        //                 contents: bytemuck::cast_slice(&self.vertices),
        //                 usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        //             }
        //         ));
        //     } else {
        //         queue.write_buffer(self.vertices_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&self.vertices));
        //     }
        //     Ok(())
        // }

        match self.vertices_buffer_id {
            Some(id) => {
                match geo_buffers.get_mut(&id) {
                    Some(geo) => {
                        geo.reset();
                        geo.update_f32(vertices, 0);
                        geo.update_buffer(device, queue);
                        Ok(())
                    },
                    None => todo!(),
                }
            },
            None => todo!(),
        }
    }

    pub fn set_indices<GBP: GeometryBufferPool<GBID>>(
        &mut self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        indices: &[u16],
        geo_buffers: &mut GBP,
    ) -> Result<(), ESpineError> {
        self.dirty_indices = true;
        // if indices.len() > self.indices.len() {
        //     // println!(">>>>>>>>>>>>>>>> I0");
        //     Err(ESpineError::MeshCanntStoreMoreThanMaxVertices)
        // } else {
        //     // println!(">>>>>>>>>>>>>>>> I1");
        //     vec_set(&mut self.indices, indices, 0);
        //     self.indices_length = indices.len() as u32;
        //     if self.indices_buffer.is_none() {
        //         self.indices_buffer = Some(device.create_buffer_init(
        //             &wgpu::util::BufferInitDescriptor {
        //                 label: None,
        //                 contents: bytemuck::cast_slice(&self.indices),
        //                 usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        //             }
        //         ));
        //     } else {
        //         queue.write_buffer(self.indices_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice(&self.indices));
        //     }
        //     Ok(())
        // }
        
        match self.indices_buffer_id {
            Some(id) => {
                match geo_buffers.get_mut(&id) {
                    Some(geo) => {
                        geo.reset();
                        geo.update_u16(indices, 0);
                        geo.update_buffer(device, queue);
                        Ok(())
                    },
                    None => todo!(),
                }
            },
            None => todo!(),
        }
    }

    pub fn draw<'a, SP: SpineShaderPool, GBP: GeometryBufferPool<GBID>, TP: TexturePool<TID>>(
        &'a self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderpass: &mut wgpu::RenderPass<'a>,
        shaders: &SP,
        textures: &TP,
        geo_buffers: &'a GBP,
    ) {
        self.material.draw(renderpass, &self.geometry, geo_buffers);
        // println!(">>>>>>>>>>>>>>>> {} >>>>> {}", self.indices_length, self.vertices_length);
        // renderpass.set_vertex_buffer(0, self.vertices_buffer.as_ref().unwrap().slice(..));
        // renderpass.set_index_buffer(self.indices_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
        // renderpass.draw_indexed(0..self.indices_length, 0, 0..1);
    }

    pub fn draw_with_offset(&self) {

    }

    pub fn bind() {

    }

    pub fn unbind() {

    }

    fn update(&mut self) {

    }

    pub fn init_material<SP: SpineShaderPool>(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        device: &wgpu::Device,
        shader: EShader,
        shader_pool: &SP
    ) {
        match shader {
            EShader::Colored => {
                SpineMaterialColored::init(mat, device, shader_pool);
            },
            EShader::ColoredTextured => {
                SpineMaterialColoredTextured::init(mat, device, shader_pool);
            },
            EShader::TwoColoredTextured => {
                SpineMaterialColoredTexturedTwo::init(mat, device, shader_pool);
            },
        }
    }
    pub fn update_uniform<TP: TexturePool<TID>, SP: SpineShaderPool>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shaders: &SP,
        textures: &TP,
    ) {
        let texture_layout = match self.shader.unwrap() {
            EShader::Colored => shaders.get_spine_shader_colored().get_texture_layout(),
            EShader::ColoredTextured => shaders.get_spine_shader_colored_textured().get_texture_layout(),
            EShader::TwoColoredTextured => shaders.get_spine_shader_colored_textured_two().get_texture_layout(),
        };
        Material::<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>::update_uniform(&mut self.material, device, queue, texture_layout, textures);
    }
    pub fn mvp_matrix(
        &mut self,
        queue: &wgpu::Queue,
        mvp: UniformKindMat4,
    ) {
        Material::<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>::mvp_matrix(&mut self.material, mvp);
    }
    pub fn mask_flag(
        &mut self,
        queue: &wgpu::Queue,
        mask_flag: UniformKindFloat4,
    ) {
        Material::<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>::mask_flag(&mut self.material, mask_flag);
    }
    pub fn texture<TP: TexturePool<TID>, SP: SpineShaderPool>(
        &mut self,
        device: &wgpu::Device,
        key: Option<TID>,
        shaders: &SP,
        textures: &TP,
    ) {
        match key {
            Some(key) => {
                Material::<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>::texture(device, &mut self.material, self.shader.unwrap(), shaders, key, textures);
            },
            None => {},
        }
        
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EVertexAttribute {
    Position,
    Textcoords,
    Color,
    Color2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    name: EVertexAttribute,
    ty: EVertexDataFormat,
    num_elements: u32,
}

impl VertexAttribute {
    pub fn elements(attributes: &Vec<VertexAttribute>) -> u32 {
        let mut result = 0;
        for v in attributes.iter() {
            result += v.num_elements;
        }

        result
    }
    pub fn position_2() -> Self {
        Self { name: EVertexAttribute::Position, ty: EVertexDataFormat::F32, num_elements: 2 }
    }
    pub fn position_3() -> Self {
        Self { name: EVertexAttribute::Position, ty: EVertexDataFormat::F32, num_elements: 3 }
    }
    pub fn texcoords() -> Self {
        Self { name: EVertexAttribute::Textcoords, ty: EVertexDataFormat::F32, num_elements: 2 }
    }
    pub fn color() -> Self {
        Self { name: EVertexAttribute::Color, ty: EVertexDataFormat::F32, num_elements: 4 }
    }
    pub fn color2() -> Self {
        Self { name: EVertexAttribute::Color2, ty: EVertexDataFormat::F32, num_elements: 4 }
    }
}

pub fn draw_mesh_colored<PP: SpinePipelinePool>(
    renderpass: &mut wgpu::RenderPass,
    pipeline_pool: &PP,
    material: &mut SpineMaterialColored,
) {

}