use std::{hash::{Hash, Hasher}, num::NonZeroU64};

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_atom::Atom;
use pi_hash::DefaultHasher;
use pi_render::{rhi::{bind_group::BindGroup, asset::{RenderRes, TextureRes}, sampler::SamplerDesc, device::RenderDevice, RenderQueue}, renderer::{sampler::SamplerRes, bind_group::BindGroupLayout}, asset::ASSET_SIZE_FOR_UNKOWN};
use pi_scene_math::{Number, Matrix};
use pi_share::Share;

use crate::{shaders::{KeySpineShader, SingleSpineBindGroupLayout}, binds::param::{SpineBindBuffer, BindBufferAllocator, BindParam}};


pub struct UsedBindGroupSet0 {
    pub bindgroup: Handle<RenderRes<BindGroup>>,
    pub offsets: [wgpu::DynamicOffset;1],
}

pub struct BindGroupSet1 {
    pub bindgroup: Handle<RenderRes<BindGroup>>,
}

pub struct UsedBindGroupSet1 {
    pub bindgroup: Handle<RenderRes<BindGroup>>,
}

#[derive(Debug, Clone)]
pub struct KeySpineBindGroup {
    url: Option<u64>,
    buffer: Handle<SpineBindBuffer>,
    sampler: Option<SamplerDesc>,
}
impl KeySpineBindGroup {
    fn to_u64(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
impl Hash for KeySpineBindGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.buffer.key().hash(state);
        self.sampler.hash(state);
    }
}
impl PartialEq for KeySpineBindGroup {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.buffer.key() == other.buffer.key() && self.sampler == other.sampler
    }
}
impl Eq for KeySpineBindGroup {}

pub struct SpineBindGroup {
    pub(crate) bindgroup: Handle<RenderRes<BindGroup>>,
    texture: Option<Handle<TextureRes>>,
    sampler: Option<Handle<SamplerRes>>,
    param: Handle<SpineBindBuffer>,
}
impl SpineBindGroup {
    pub fn colored(
        param: Handle<SpineBindBuffer>,
        device: &RenderDevice,
        asset_mgr: &Share<AssetMgr<RenderRes<BindGroup>>>,
        bind_group_layouts: &SingleSpineBindGroupLayout,
    ) -> Self {
        let key_layout = KeySpineShader::Colored;

        let key = KeySpineBindGroup {
            url: None,
            buffer: param.clone(),
            sampler: None,
        };
        let key_u64 = key.to_u64();

        let bindgroup = if let Some(bindgroup) = asset_mgr.get(&key_u64) {
            bindgroup
        } else {
            let bindgroup = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layouts.colored,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: param.buffer(), offset: 0, size: NonZeroU64::new(BindParam::SIZE as u64) } ),
                        }
                    ],
                }
            );

            asset_mgr.insert(key_u64, RenderRes::new(bindgroup, ASSET_SIZE_FOR_UNKOWN)).unwrap()
        };

        Self { bindgroup, texture: None, sampler: None, param }
    }
    pub fn colored_textured(
        param: Handle<SpineBindBuffer>,
        device: &RenderDevice,
        texture: Handle<TextureRes>,
        sampler: Handle<SamplerRes>,
        asset_mgr: &Share<AssetMgr<RenderRes<BindGroup>>>,
        bind_group_layouts: &SingleSpineBindGroupLayout,
    ) -> Self {
        let key = KeySpineBindGroup {
            url: Some(texture.key().clone()),
            buffer: param.clone(),
            sampler: Some(sampler.key().clone()),
        };
        let key_u64 = key.to_u64();

        let bindgroup = if let Some(bindgroup) = asset_mgr.get(&key_u64) {
            bindgroup
        } else {
            let bindgroup = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layouts.colored_textured,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: param.buffer(), offset: 0, size: NonZeroU64::new(BindParam::SIZE as u64) } ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&sampler.0),
                        },
                    ],
                }
            );

            asset_mgr.insert(key_u64, RenderRes::new(bindgroup, ASSET_SIZE_FOR_UNKOWN)).unwrap()
        };

        Self { bindgroup, texture: Some(texture), sampler: Some(sampler), param }
    }
    pub fn two_colored_textured(
        param: Handle<SpineBindBuffer>,
        device: &RenderDevice,
        texture: Handle<TextureRes>,
        sampler: Handle<SamplerRes>,
        asset_mgr: &Share<AssetMgr<RenderRes<BindGroup>>>,
        bind_group_layouts: &SingleSpineBindGroupLayout,
    ) -> Self {
        Self::colored_textured(param, device, texture, sampler, asset_mgr, bind_group_layouts)
    }
}