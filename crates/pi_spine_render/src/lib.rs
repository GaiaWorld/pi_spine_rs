
use std::{mem::replace, num::NonZeroU32};

use bevy::prelude::{ResMut, Resource, App, Plugin, Res, CoreStage};
use futures::FutureExt;
use pi_assets::{mgr::AssetMgr, asset::{Handle, GarbageEmpty}};
use pi_atom::Atom;
use pi_bevy_asset::ShareAssetMgr;
use pi_bevy_render_plugin::{PiRenderDevice, PiRenderQueue, node::Node, PiSafeAtlasAllocator, SimpleInOut, PiScreenTexture, PiClearOptions, PiRenderGraph};
use pi_hash::XHashMap;
use pi_render::{rhi::{sampler::{SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}, asset::TextureRes}, asset::TAssetKeyU64, renderer::{sampler::SamplerRes, draw_obj_list::DrawList}, components::view::target_alloc::{ShareTargetView, TargetDescriptor, TextureDescriptor}};
use renderer::{RendererAsync, SpineResource};
use shaders::KeySpineShader;
use smallvec::SmallVec;


pub mod binds;
pub mod bind_groups;
pub mod shaders;
pub mod vertex_buffer;
pub mod renderer;
pub mod ecs;

pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const SAMPLER_DESC: SamplerDesc = SamplerDesc {
    address_mode_u: EAddressMode::Repeat,
    address_mode_v: EAddressMode::Repeat,
    address_mode_w: EAddressMode::Repeat,
    mag_filter: EFilterMode::Linear,
    min_filter: EFilterMode::Linear,
    mipmap_filter: EFilterMode::Nearest,
    compare: None,
    anisotropy_clamp: EAnisotropyClamp::One,
    border_color: None,
};

#[derive(Debug, Clone, Copy)]
pub struct KeySpineRenderer(pub(crate) u32);

pub struct SpineRenderNodeParam {
    render: RendererAsync,
    width: u32,
    height: u32,
    screen: bool,
}

pub struct SpineRenderNode(KeySpineRenderer);
impl Node for SpineRenderNode {
    type Input = ();

    type Output = SimpleInOut;

    type Param = Res<'static, PiScreenTexture>;

    fn run<'a>(
        &'a mut self,
        world: &'a bevy::prelude::World,
        param: &'a mut bevy::ecs::system::SystemState<Self::Param>,
        _context: pi_bevy_render_plugin::RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        _input: &'a Self::Input,
        _usage: &'a pi_bevy_render_plugin::node::ParamUsage,
    ) -> pi_futures::BoxFuture<'a, Result<Self::Output, String>> {
        let atlas_allocator = world.get_resource::<PiSafeAtlasAllocator>().unwrap();
        let temp: Vec<ShareTargetView> = vec![];
        
        let spine_ctx = world.get_resource::<SpineRenderContext>().unwrap();

        let renderer = if let Some(renderer) = spine_ctx.list.get(&self.0.0) {
            renderer
        } else {
            return async move {
                Ok(SimpleInOut { target: None })
            }.boxed();
        };
        
        if renderer.screen == false {
            Box::pin(
            async move {
                
                let mut encoder = commands.0.as_ref().borrow_mut();

                let target_type = atlas_allocator.get_or_create_type(
                    TargetDescriptor {
                        texture_descriptor: SmallVec::from_slice(
                            &[
                                TextureDescriptor {
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: FORMAT,
                                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                                    base_mip_level: 0,
                                    base_array_layer: 0,
                                    array_layer_count: None,
                                    view_dimension: None,
                                }
                            ]
                        ),
                        need_depth: false,
                        default_width: 2048,
                        default_height: 2048,
                    }
                );

                let target = atlas_allocator.allocate(renderer.width, renderer.height, target_type, temp.iter());
                
                {
                    let mut renderpass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("RenderNode"),
                            color_attachments: &[
                                Some(
                                    wgpu::RenderPassColorAttachment {
                                        view: &target.target().colors[0].0,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: true,
                                        }
                                    }
                                )
                            ],
                            depth_stencil_attachment: None,
                        }
                    );
                    let rect = target.rect();
                    let (x, y, w, h) = (
                        rect.min.x as u32,
                        rect.min.y as u32,
                        (rect.max.x - rect.min.x) as u32,
                        (rect.max.y - rect.min.y) as u32,
                    );
            
                    let min_depth = 0.;
                    let max_depth = 1.;
            
                    renderpass.set_viewport(x as f32, y as f32, w as f32, h as f32, min_depth, max_depth);
                    renderpass.set_scissor_rect(x as u32, y as u32, w as u32, h as u32);
                    log::warn!("Draws: {:?}", renderer.render.drawobjs.list.len());
                    DrawList::render(renderer.render.drawobjs.list.as_slice(), &mut renderpass);
                }

                Ok(SimpleInOut { target: Some(target) })
            })
        } else {
            let screen = param.get(world);
            let view = screen.0.as_ref().unwrap().view.as_ref().unwrap().clone();

            Box::pin(async move {
                let mut encoder = commands.0.as_ref().borrow_mut();
                let mut renderpass = encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[
                            Some(
                                wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: true,
                                    }
                                }
                            )
                        ],
                        depth_stencil_attachment: None,
                    }
                );
                
                // renderpass.set_viewport(x, y, w, h, min_depth, max_depth);
                // renderpass.set_scissor_rect(x as u32, y as u32, w as u32, h as u32);
                log::info!("Draws: {:?}", renderer.render.drawobjs.list.len());
                DrawList::render(renderer.render.drawobjs.list.as_slice(), &mut renderpass);

                Ok(SimpleInOut { target: None })
            })
        }
    }
}


#[derive(Resource)]
pub struct SpineRenderContext {
    counter: u32,
    list: XHashMap<u32, SpineRenderNodeParam>,
}
impl SpineRenderContext {
    pub fn new() -> Self {
        Self { counter: 0, list: XHashMap::default() }
    }
    pub fn create_renderer(&mut self, screen: bool) -> KeySpineRenderer {
        self.counter += 1;
        let id = self.counter;

        let render = SpineRenderNodeParam { render: RendererAsync::new(), width: 128, height: 128, screen };
        self.list.insert(id, render);

        KeySpineRenderer(id)
    }
}

#[derive(Clone)]
pub enum ESpineCommand {
    Reset(KeySpineRenderer),
    RenderSize(KeySpineRenderer, u32, u32),
    Shader(KeySpineRenderer, Option<KeySpineShader>),
    UseTexture(KeySpineRenderer, Option<u64>),
    Texture(KeySpineRenderer, u64, Handle<TextureRes>, SamplerDesc, Handle<SamplerRes>),
    Blend(KeySpineRenderer, bool),
    BlendMode(KeySpineRenderer, wgpu::BlendFactor, wgpu::BlendFactor),
    Uniform(KeySpineRenderer, Vec<f32>),
    Draw(KeySpineRenderer, Vec<f32>, Vec<u16>, u32, u32),
}



#[derive(Resource, Default)]
pub struct SingleSpineCommands(pub Vec<ESpineCommand>);

pub struct SysSpineCommands;
impl SysSpineCommands {
    fn sys(
        mut cmds: ResMut<SingleSpineCommands>,
        mut clearopt: ResMut<PiClearOptions>,
        mut renderers: ResMut<SpineRenderContext>,
    ) {
        clearopt.color.g = 0.;
        let mut list = replace(&mut cmds.0, vec![]);
        list.drain(..).for_each(|cmd| {
            match cmd {
                ESpineCommand::Uniform(id, val) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        // log::warn!("Cmd: Uniform");
                        renderer.render.uniform(val);
                    }
                },
                ESpineCommand::Shader(id, val) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        // log::warn!("Cmd: Shader");
                        renderer.render.shader(val);
                    }
                },
                ESpineCommand::UseTexture(id, val) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        let samplerdesc = SAMPLER_DESC.clone();
                        // log::warn!("Cmd: UseTexture");
                        renderer.render.texture(val, Some(samplerdesc));
                    }
                },
                ESpineCommand::Draw(id, vertices, indices, vlen, ilen) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        // log::warn!("Cmd: Draw");
                        renderer.render.draw(vertices, Some(indices), vlen, ilen);
                    }
                },
                ESpineCommand::Texture(id, key, value, key2, value2) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        // log::warn!("Cmd: Texture");
                        renderer.render.textures.insert(key, value);
                        renderer.render.samplers.insert(key2, value2);
                    }
                },
                ESpineCommand::RenderSize(id, width, height) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        renderer.width = width;
                        renderer.height = height;
                    }
                },
                ESpineCommand::Reset(id) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        renderer.render.reset();
                    }
                },
                ESpineCommand::Blend(id, val) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        renderer.render.blend(val);
                    }
                }
                ,
                ESpineCommand::BlendMode(id, val0, val1) => {
                    if let Some(renderer) = renderers.list.get_mut(&id.0) {
                        renderer.render.blend_mode(val0, val1);
                    }
                },
            }
        })
    }
}

pub struct SysSpineRendererApply;
impl SysSpineRendererApply {
    fn sys(
        mut renderers: ResMut<SpineRenderContext>,
        mut resource: ResMut<SpineResource>,
        device: Res<PiRenderDevice>,
        queue: Res<PiRenderQueue>,
        asset_samplers: Res<ShareAssetMgr<SamplerRes>>,
        asset_textures: Res<ShareAssetMgr<TextureRes>>,
    ) {
        // log::warn!("Apply: {:?}", renderers.list.len());
        renderers.list.iter_mut().for_each(|(_, v)| {
            v.render.drawlist(&device, &queue, &mut resource, &asset_samplers, &asset_textures);
        })
    }
}

// pub trait TInterfaceSpine: TShell {
//     fn create_spine_renderer(&mut self, name: Atom, next_node: Option<Atom>) -> KeySpineRenderer;
//     fn dispose_spine_renderer(&mut self, id_renderer: KeySpineRenderer) -> &mut Self;
//     fn spine_reset(&mut self, id_renderer: KeySpineRenderer) -> &mut Self;
//     fn spine_uniform(&mut self, id_renderer: KeySpineRenderer, value: &[f32]) -> &mut Self;
//     fn spine_shader(&mut self, id_renderer: KeySpineRenderer, value: KeySpineShader) -> &mut Self;
//     fn spine_use_texture(&mut self, id_renderer: KeySpineRenderer, value: u64) -> &mut Self;
//     fn spine_draw(&mut self, id_renderer: KeySpineRenderer, vertices: &[f32], indices: &[u16], vlen: u32, ilen: u32) -> &mut Self;
//     fn spine_texture(&mut self, id_renderer: KeySpineRenderer, key: Atom, data: &[u8], width: u32, height: u32) -> &mut Self;
// }

pub trait TInterfaceSpine: pi_bevy_ecs_extend::TShell {
    fn create_spine_renderer(
        name: Atom,
        next_node: Option<Atom>,
        ctx: &mut SpineRenderContext,
        render_graph: &mut PiRenderGraph,
    ) -> KeySpineRenderer {
        
        let id = ctx.create_renderer(next_node.is_none());

        let key = String::from(name.as_str());
        match render_graph.add_node(key.clone(), SpineRenderNode(id)) {
            Ok(v) => {
                if let Some(next_node) = next_node {
                    render_graph.add_depend(key, String::from(next_node.as_str()));
                    ctx.list.get_mut(&id.0).unwrap().render.target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
                } else {
                    render_graph.set_finish(key, true);
                }
        
                render_graph.dump_graphviz();
            },
            Err(e) => {
                log::warn!("Add Node Error {:?}", e)
            },
        }

        id
    }

    fn dispose_spine_renderer(
        id_renderer: KeySpineRenderer,
        ctx: &mut SpineRenderContext,
    ) {
        ctx.list.remove(&id_renderer.0);
    }

    fn spine_uniform(
        cmds: &mut Vec<ESpineCommand>,
        id_renderer: KeySpineRenderer,
        value: &[f32],
    ) {
        cmds.push(ESpineCommand::Uniform(id_renderer, value.to_vec()));
    }

    fn spine_shader(
        cmds: &mut Vec<ESpineCommand>,
        id_renderer: KeySpineRenderer,
        value: KeySpineShader,
    ) {
        cmds.push(ESpineCommand::Shader(id_renderer, Some(value)));
    }

    fn spine_use_texture(
        cmds: &mut Vec<ESpineCommand>,
        id_renderer: KeySpineRenderer,
        value: u64,
    ) {
        cmds.push(ESpineCommand::UseTexture(id_renderer, Some(value)));
    }

    fn spine_draw(
        cmds: &mut Vec<ESpineCommand>,
        id_renderer: KeySpineRenderer,
        vertices: &[f32],
        indices: &[u16],
        vlen: u32,
        ilen: u32,
    ) {
        cmds.push(ESpineCommand::Draw(id_renderer, vertices.to_vec(), indices.to_vec(), vlen, ilen));
    }

    fn spine_texture(
        cmds: &mut Vec<ESpineCommand>,
        id_renderer: KeySpineRenderer,
        key: Atom,
        data: &[u8],
        width: u32,
        height: u32,
        device: & PiRenderDevice,
        queue: & PiRenderQueue,
        asset_textures: & ShareAssetMgr<TextureRes>,
        asset_samplers: & ShareAssetMgr<SamplerRes>,
    ) {

        let key_u64 = key.asset_u64();
        let texture = if let Some(textureres) = asset_textures.get(&key_u64) {
            textureres
        } else {
            let texture = (***device).create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            });
            queue.write_texture(
                // Tells wgpu where to copy the pixel data
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                // The actual pixel data
                data,
                // The layout of the texture
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * width),
                    rows_per_image: std::num::NonZeroU32::new(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count:  NonZeroU32::new(0),
                base_array_layer: 0,
                array_layer_count: None,
            });

            let textureres = TextureRes::new(width, height, (width * height * 4) as usize, texture_view, true);
            
            if let Some(texture) = asset_textures.insert(key_u64, textureres) {
                texture
            } else {
                return;
            }
        };

        let samplerdesc = SAMPLER_DESC.clone();

        let sampler = if let Some(sampler) = asset_samplers.get(&samplerdesc) {
            sampler
        } else {
            if let Some(sampler) = asset_samplers.insert(samplerdesc.clone(), SamplerRes::new(&device, &samplerdesc)) {
                sampler
            } else {
                return;
            }
        };

        cmds.push(ESpineCommand::Texture(id_renderer, key_u64, texture, samplerdesc, sampler));
    }

    fn spine_reset(
        cmds: &mut Vec<ESpineCommand>,
        id_renderer: KeySpineRenderer,
    ) {
        cmds.push(ESpineCommand::Reset(id_renderer));
    }
}

#[derive(Default)]
pub struct PluginSpineRenderer;
impl Plugin for PluginSpineRenderer {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<ShareAssetMgr<SamplerRes>>().is_none() {
            app.insert_resource(ShareAssetMgr(AssetMgr::<SamplerRes>::new(GarbageEmpty(), false, 32 * 1024, 30 * 1000)));
        }
        if app.world.get_resource::<ShareAssetMgr<TextureRes>>().is_none() {
            app.insert_resource(ShareAssetMgr(AssetMgr::<TextureRes>::new(GarbageEmpty(), false, 32 * 1024 * 1024, 30 * 1000)));
        }
        
        let device = app.world.get_resource::<PiRenderDevice>().unwrap().0.clone();
        app.insert_resource(SingleSpineCommands::default())
            .insert_resource(SpineResource::new(&device))
            .insert_resource(SpineRenderContext::new());

        app.add_system_to_stage(CoreStage::First, SysSpineCommands::sys);
        app.add_system_to_stage(CoreStage::Update, SysSpineRendererApply::sys);

        log::warn!("PluginSpineRenderer");
    }
}