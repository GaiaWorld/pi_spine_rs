use std::hash;

use render_geometry::{geometry::{GeometryBufferDesc},};
use render_material::{material::{Material, MaterialUniformDesc, MaterialAttributeDesc, EUniformDataFormat, UnifromData, MaterialTextureDesc, UniformKindFloat2}, texture::{MaterialTextureSampler, }};
use render_data_container::{Matrix, Vector4, TextureID, TexturePool, EVertexDataFormat};

use crate::shaders::{SpineShaderPool, EShader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, hash::Hash)]
pub enum SpineVertexBufferKindKey {
    Vertices,
    Indices,
}
impl render_data_container::TVertexBufferKindKey for SpineVertexBufferKindKey {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, hash::Hash)]
pub enum SpineMaterialBlockKindKey{
    Vertices,
    UniformSet0,
    MVPMatrix,
    MaskFlag,
    Visibility,
    Texture,
}
impl render_data_container::TMaterialBlockKindKey for SpineMaterialBlockKindKey {}


pub struct SpineMaterialColored {}
impl SpineMaterialColored {
    const A_VERTICE: GeometryBufferDesc<SpineVertexBufferKindKey> = GeometryBufferDesc { slot: 0, format: EVertexDataFormat::F32, kind: SpineVertexBufferKindKey::Vertices, size_per_vertex: 8 };
    const U_MVP_MATRIX: MaterialUniformDesc<SpineMaterialBlockKindKey> = MaterialUniformDesc { kind: SpineMaterialBlockKindKey::MVPMatrix, format: EUniformDataFormat::Mat4, bind: 0 };
    const U_MASK_FLAG: MaterialUniformDesc<SpineMaterialBlockKindKey> = MaterialUniformDesc { kind: SpineMaterialBlockKindKey::MaskFlag, format: EUniformDataFormat::Float4, bind: 0 };

    pub fn init<SP: SpineShaderPool, TID: TextureID>(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        device: &wgpu::Device,
        spine_shader_pool: &SP
    ) {
        mat.init(
            device,
            vec![
                Self::A_VERTICE
            ],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            vec![
                SpineMaterialColored::U_MVP_MATRIX,
                SpineMaterialColored::U_MASK_FLAG,
            ],
            vec![],
            &spine_shader_pool.get_spine_shader_colored().get_uniform_layout()
        );
    }
}


pub struct SpineMaterialColoredTextured {}
impl SpineMaterialColoredTextured {
    const A_VERTICE: GeometryBufferDesc<SpineVertexBufferKindKey> = GeometryBufferDesc { slot: 0, format: EVertexDataFormat::F32, kind: SpineVertexBufferKindKey::Vertices, size_per_vertex: 8 };
    const VISIBILITY: MaterialUniformDesc<SpineMaterialBlockKindKey> = MaterialUniformDesc { kind: SpineMaterialBlockKindKey::Visibility, format: EUniformDataFormat::Float2, bind: 0 };
    const TEXTURE: MaterialTextureDesc<SpineMaterialBlockKindKey> = MaterialTextureDesc { kind: SpineMaterialBlockKindKey::Texture, bind: 1, bind_sampler: 0 };
    pub fn init<SP: SpineShaderPool, TID: TextureID>(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        device: &wgpu::Device,
        spine_shader_pool: &SP
    ) {
        mat.init(
            device,
            vec![
                Self::A_VERTICE
            ],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            vec![
                SpineMaterialColored::U_MVP_MATRIX,
                SpineMaterialColored::U_MASK_FLAG,
                SpineMaterialColoredTextured::VISIBILITY,
            ],
            vec![
                SpineMaterialColoredTextured::TEXTURE
                // SpineMaterialBlockKindKey::Texture
            ],
            &spine_shader_pool.get_spine_shader_colored_textured().get_uniform_layout()
        );
    }
    pub fn texture<TID: TextureID, TP: TexturePool<TID>, SP: SpineShaderPool>(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        device: &wgpu::Device,
        key: TID,
        textures: &TP,
        spine_shader_pool: &SP,
    ) {
        let kind = SpineMaterialColoredTextured::TEXTURE;
        let layout = spine_shader_pool.get_spine_shader_colored_textured().get_texture_layout();
        match layout {
            Some(layout) => {
                mat.set_texture(
                    kind,
                    &MaterialTextureSampler::new(wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Linear),
                    key
                );
            },
            None => {},
        }
    }
}

pub struct SpineMaterialColoredTexturedTwo {}
impl SpineMaterialColoredTexturedTwo {
    const A_VERTICE: GeometryBufferDesc<SpineVertexBufferKindKey> = GeometryBufferDesc { slot: 0, format: EVertexDataFormat::F32, kind: SpineVertexBufferKindKey::Vertices, size_per_vertex: 12 };
    pub fn init<SP: SpineShaderPool, TID: TextureID>(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        device: &wgpu::Device,
        spine_shader_pool: &SP,
    ) {
        mat.init(
            device,
            vec![
                Self::A_VERTICE
            ],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            vec![
                SpineMaterialColored::U_MVP_MATRIX,
                SpineMaterialColored::U_MASK_FLAG,
                SpineMaterialColoredTextured::VISIBILITY,
            ],
            vec![
                SpineMaterialColoredTextured::TEXTURE
            ],
            &spine_shader_pool.get_spine_shader_colored_textured_two().get_uniform_layout()
        );
    }
    pub fn texture<TID: TextureID, TP: TexturePool<TID>, SP: SpineShaderPool>(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        device: &wgpu::Device,
        key: TID,
        textures: &TP,
        spine_shader_pool: &SP,
    ) {
        let kind = SpineMaterialColoredTextured::TEXTURE;
        let layout = &spine_shader_pool.get_spine_shader_colored_textured_two().get_texture_layout();
        match layout {
            Some(layout) => {
                mat.set_texture(
                    kind,
                    &MaterialTextureSampler::new(wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Linear),
                    key,
                );
            },
            None => {},
        }
    }
}

pub trait TSpineMaterialUpdate<TID: TextureID> {
    fn mvp_matrix(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        matrix: Matrix
    ) {
        mat.set_uniform(
            SpineMaterialColored::U_MVP_MATRIX,
            UnifromData::Mat4(matrix)
        );
    }
    fn mask_flag(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        data: Vector4
    ) {
        mat.set_uniform(
            SpineMaterialColored::U_MASK_FLAG,
            UnifromData::Float4(data)
        );
    }
    fn visibility(
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        data: UniformKindFloat2,
    ) {
        mat.set_uniform(
            SpineMaterialColoredTextured::VISIBILITY,
            UnifromData::Float2(data)
        );
    }
    fn texture<TP: TexturePool<TID>, SP: SpineShaderPool>(
        device: &wgpu::Device,
        mat: &mut Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID>,
        shader: EShader,
        shaders: &SP,
        key: TID,
        textures: &TP,
    ) {
        match shader {
            EShader::Colored => {},
            EShader::ColoredTextured => {
                SpineMaterialColoredTextured::texture(mat, device, key, textures, shaders);
            },
            EShader::TwoColoredTextured => {
                SpineMaterialColoredTexturedTwo::texture(mat, device, key, textures, shaders);
            },
        }
        // mat.set_texture_2d(
        //     device,
        //     kind,
        //     layout,
        //     sampler,
        //     key,
        //     textures
        // );
    }
}

impl<TID: TextureID> TSpineMaterialUpdate<TID> for Material<SpineVertexBufferKindKey, SpineMaterialBlockKindKey, TID> {}