
use std::mem::replace;

use bevy_ecs::{prelude::{Query, ResMut, Resource, Res, IntoSystemConfigs, Entity, Commands, SystemSet, apply_deferred}, world::World, system::SystemState};
use bevy_app::prelude::{Update, App, Plugin};
use crossbeam::queue::SegQueue;
use futures::FutureExt;
use pi_assets::{mgr::{AssetMgr, LoadResult}, asset::{Handle, GarbageEmpty}};
use pi_async_rt::prelude::AsyncRuntime;
use pi_atom::Atom;
use pi_bevy_asset::ShareAssetMgr;
use pi_bevy_render_plugin::{
    PiRenderDevice, PiRenderQueue, node::{Node, ParamUsage}, PiSafeAtlasAllocator, SimpleInOut, PiClearOptions, PiRenderGraph, NodeId, GraphError, PiRenderSystemSet, render_cross::GraphId, PiRenderOptions,
    constant::texture_sampler::*, RenderContext
};
use pi_null::Null;
// use pi_window_renderer::WindowRenderer;
use pi_hal::{runtime::RENDER_RUNTIME, loader::AsyncLoader};
use pi_hash::XHashMap;
use pi_render::{rhi::{sampler::{SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}, asset::{TextureRes, ImageTextureDesc}}, asset::TAssetKeyU64, renderer::{sampler::SamplerRes, draw_obj_list::DrawList}, components::view::target_alloc::{ShareTargetView, TargetDescriptor, TextureDescriptor}};
use pi_share::Share;
use renderer::{RendererAsync, SpineResource};
use shaders::KeySpineShader;
use smallvec::SmallVec;
use wgpu::StoreOp;


pub mod binds;
pub mod bind_groups;
pub mod shaders;
pub mod vertex_buffer;
pub mod renderer;
pub mod ecs;

pub const FORMAT: ColorFormat = ColorFormat::Rgba8Unorm;
pub const SAMPLER_DESC: SamplerDesc = SamplerDesc {
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct KeySpineRenderer(pub Entity);
impl KeySpineRenderer {
    pub fn from_f64(val: f64) -> Self {
        Self(Entity::from_bits(val.to_bits()))
    }
}

pub struct SpineRenderNodeParam {
    render: RendererAsync,
    width: u32,
    height: u32,
    to_screen: bool,
}
impl SpineRenderNodeParam {
    pub fn render_mut(&mut self) -> &mut RendererAsync {
        &mut self.render
    }
}

pub struct SpineRenderNode{
	pub renderer: KeySpineRenderer,
	rt: Option<ShareTargetView>,
}

impl SpineRenderNode {
	pub fn new(renderer: KeySpineRenderer) -> Self {
		Self { renderer, rt: None }
	}
}

impl Node for SpineRenderNode {
    type Input = ();

    type Output = SimpleInOut;

	type BuildParam = ();
    type RunParam = ();

	fn build<'a>(
        &'a mut self,
        world: &'a mut World,
        param: &'a mut SystemState<Self::BuildParam>,
        context: RenderContext,
		input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId,
		from: &'a [NodeId],
		to: &'a [NodeId],
    ) -> Result<Self::Output, String> {
		let spine_ctx = world.get_resource::<SpineRenderContext>().unwrap();
		let renderer = if let Some(renderer) = spine_ctx.list.get(&self.renderer) {
			renderer
		} else {
			log::warn!("SpineGraph:: None renderer");
			return Ok(SimpleInOut { target: None, valid_rect: None })
		};

		if renderer.to_screen == false {
			let temp: Vec<ShareTargetView> = vec![];
			let atlas_allocator = world.get_resource::<PiSafeAtlasAllocator>().unwrap();
			let target_type = atlas_allocator.get_or_create_type(
				TargetDescriptor {
					colors_descriptor: SmallVec::from_slice(
						&[
							TextureDescriptor {
								mip_level_count: 1,
								sample_count: 1,
								dimension: wgpu::TextureDimension::D2,
								format: FORMAT.val(),
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
					depth_descriptor: None,
				}
			);

			let target = atlas_allocator.allocate(renderer.width, renderer.height, target_type, temp.iter());
			self.rt = Some(target.clone());
			Ok(SimpleInOut { target: Some(target), valid_rect: None })
		} else {
			Ok(SimpleInOut { target: None, valid_rect: None })
		}
	}

    fn run<'a>(
        &'a mut self,
        world: &'a bevy_ecs::prelude::World,
        param: &'a mut bevy_ecs::system::SystemState<Self::RunParam>,
        _context: pi_bevy_render_plugin::RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        _input: &'a Self::Input,
        _usage: &'a pi_bevy_render_plugin::node::ParamUsage,
		_id: NodeId,
		_from: &[NodeId],
		_to: &[NodeId],
    ) -> pi_futures::BoxFuture<'a, Result<(), String>> {
        let atlas_allocator = world.get_resource::<PiSafeAtlasAllocator>().unwrap();
        let temp: Vec<ShareTargetView> = vec![];
        
        let spine_ctx = world.get_resource::<SpineRenderContext>().unwrap();

        let renderer = if let Some(renderer) = spine_ctx.list.get(&self.renderer) {
            renderer
        } else {
            return async move {
                log::warn!("SpineGraph:: None renderer");
                Ok(())
            }.boxed();
        };

		return Box::pin(
            async move {
                
                let mut encoder = commands.0.as_ref().borrow_mut();


                let target = match &self.rt {
                    Some(r) => r,
                    None => return  Ok(()),
                };
                
                {
                    let _renderpass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("RenderNode"),
                            color_attachments: &[
                                Some(
                                    wgpu::RenderPassColorAttachment {
                                        view: &target.target().colors[0].0,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0., g: 0., b: 0., a: 0. }),
                                            store: StoreOp::Store,
                                        }
                                    }
                                )
                            ],
                            depth_stencil_attachment: None,
							timestamp_writes: None,
                            occlusion_query_set: None,
                        }
                    );
                }
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
                                            store: StoreOp::Store,
                                        }
                                    }
                                )
                            ],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
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
                    // log::warn!("SpineGraph DrawList::render: {:?}", renderer.render.drawobjs.list.len());
                    DrawList::render(renderer.render.drawobjs.list.as_slice(), &mut renderpass);
                }

                Ok(())
        	});
        
		// let screen = param.get(world);

		// Box::pin(async move {
		//     if let Some(view) = screen.view() {
		//         let mut encoder = commands.0.as_ref().borrow_mut();
		//         let mut renderpass = encoder.begin_render_pass(
		//             &wgpu::RenderPassDescriptor {
		//                 label: None,
		//                 color_attachments: &[
		//                     Some(
		//                         wgpu::RenderPassColorAttachment {
		//                             view: &view,
		//                             resolve_target: None,
		//                             ops: wgpu::Operations {
		//                                 load: wgpu::LoadOp::Load,
		//                                 store: true,
		//                             }
		//                         }
		//                     )
		//                 ],
		//                 depth_stencil_attachment: None,
		//             }
		//         );
				
		//         // renderpass.set_viewport(x, y, w, h, min_depth, max_depth);
		//         // renderpass.set_scissor_rect(x as u32, y as u32, w as u32, h as u32);
		//         // log::warn!("SpineGraph Draws: {:?}", renderer.render.drawobjs.list.len());
		//         DrawList::render(renderer.render.drawobjs.list.as_slice(), &mut renderpass);
		//     }

		//     Ok(SimpleInOut { target: None, valid_rect: None })
		// })
    }
}


#[derive(Resource)]
pub struct SpineRenderContext {
    list: XHashMap<KeySpineRenderer, SpineRenderNodeParam>,
}
impl SpineRenderContext {
    pub fn new() -> Self {
        Self { list: XHashMap::default() }
    }
    pub fn get_mut(&mut self, key: KeySpineRenderer) -> Option<&mut SpineRenderNodeParam> {
        self.list.get_mut(&key)
    }
    pub fn create_renderer(&mut self, key: KeySpineRenderer, to_screen: bool) {
        // self.counter += 1;
        // let id = self.counter;

        let render = SpineRenderNodeParam { render: RendererAsync::new(), width: 128, height: 128, to_screen };
        self.list.insert(key, render);
    }
}

#[derive(Clone)]
pub enum ESpineCommand {
    Create(KeySpineRenderer, String, Option<(u32, u32)>, wgpu::TextureFormat),
    Dispose(KeySpineRenderer),
    TextureLoad(Atom),
    TextureRecord(KeySpineRenderer, Handle<TextureRes>),
    SamplerRecord(KeySpineRenderer, SamplerDesc, Handle<SamplerRes>),
    RemoveTextureRecord(KeySpineRenderer, u64),
    Reset(KeySpineRenderer),
    RenderSize(KeySpineRenderer, u32, u32),
    Shader(KeySpineRenderer, Option<KeySpineShader>),
    UseTexture(KeySpineRenderer, Option<Handle<TextureRes>>, Option<Handle<SamplerRes>>),
    Texture(KeySpineRenderer, u64, Handle<TextureRes>, SamplerDesc, Handle<SamplerRes>),
    Blend(KeySpineRenderer, bool),
    BlendMode(KeySpineRenderer, wgpu::BlendFactor, wgpu::BlendFactor),
    Uniform(KeySpineRenderer, Vec<f32>),
    Draw(KeySpineRenderer, Vec<f32>, Vec<u16>, u32, u32),
    Graph(KeySpineRenderer, NodeId),
}


pub type ActionListSpine = pi_bevy_ecs_extend::action::ActionList<ESpineCommand>;

pub fn sys_spine_cmds(
    mut cmds: ResMut<ActionListSpine>,
    mut clearopt: ResMut<PiClearOptions>,
    mut renderers: ResMut<SpineRenderContext>,
    renderopt: Res<PiRenderOptions>,
    mut graphic: ResMut<PiRenderGraph>,
    mut texloader: ResMut<SpineTextureLoad>,
    nodes: Query<&GraphId>,
    mut commands: Commands,
) {
    clearopt.color.g = 0.;
    let mut list = cmds.drain();
    // let len = list.len();
    let mut index = 0;
    list.drain(..).for_each(|cmd| {
        index += 1;
        match cmd {
            ESpineCommand::Create(id, name, rendersize, format) => {
                ActionSpine::create_spine_renderer(id, rendersize, &mut renderers, format);
                match ActionSpine::spine_renderer_apply(id, pi_atom::Atom::from(name), rendersize.is_none(), &mut graphic) {
                    Ok(nodeid) => {
                        if let Some(mut cmd) = commands.get_entity(id.0) {
                            cmd.insert(GraphId(nodeid));
                        }
                    },
                    Err(e) => {
                        log::warn!("Spine render_graph Err {:?}", e);
                    },
                }
            },
            ESpineCommand::Dispose(id_renderer) => {
                if let Ok(nodeid) = nodes.get(id_renderer.0) {
                    graphic.remove_node(nodeid.0);
                }
                ActionSpine::dispose_spine_renderer(id_renderer, &mut renderers);
                if let Some(mut cmds) = commands.get_entity(id_renderer.0) {
                    cmds.despawn();
                }
            },
            ESpineCommand::TextureLoad(key) => {
                texloader.load(key);
            },
            ESpineCommand::TextureRecord(id_renderer, val) => {
                if let Some(renderer) = renderers.get_mut(id_renderer) {
                    // log::warn!("Cmd: Texture");
                    let key_u64 = val.key();
                    renderer.render_mut().record_texture(*key_u64, val);
                }
            },
            ESpineCommand::SamplerRecord(id_renderer, samplerdesc, val) => {
                if let Some(renderer) = renderers.get_mut(id_renderer) {
                    // log::warn!("Cmd: Texture");
                    renderer.render_mut().record_sampler(samplerdesc, val);
                }
            },
            ESpineCommand::RemoveTextureRecord(id_renderer, key) => {
                if let Some(renderer) = renderers.get_mut(id_renderer) {
                    // log::warn!("Cmd: Texture");
                    renderer.render_mut().remove_texture(key);
                }
            },
            ESpineCommand::Uniform(id, val) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    // log::warn!("Cmd: Uniform");
                    renderer.render.uniform(val);
                }
            },
            ESpineCommand::Shader(id, val) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    // log::warn!("Cmd: Shader");
                    renderer.render.shader(val);
                }
            },
            ESpineCommand::UseTexture(id, val, sampler) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    // log::warn!("Cmd: UseTexture");
                    renderer.render.texture(val, sampler);
                }
            },
            ESpineCommand::Draw(id, vertices, indices, vlen, ilen) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    // log::warn!("Cmd: Draw");
                    // log::warn!("Cmd Draw: {:?} in {:?}", index, len);
                    renderer.render.draw(vertices, Some(indices), vlen, ilen, &renderopt);
                }
            },
            ESpineCommand::Texture(id, key, value, key2, value2) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    // log::warn!("Cmd: Texture");
                    renderer.render.textures.insert(key, value);
                    renderer.render.samplers.insert(key2, value2);
                }
            },
            ESpineCommand::RenderSize(id, width, height) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    renderer.width = width;
                    renderer.height = height;
                }
            },
            ESpineCommand::Reset(id) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    // log::warn!("Cmd Reset: {:?} in {:?}", index, len);
                    renderer.render.reset();
                }
            },
            ESpineCommand::Blend(id, val) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    renderer.render.blend(val);
                }
            }
            ,
            ESpineCommand::BlendMode(id, val0, val1) => {
                if let Some(renderer) = renderers.list.get_mut(&id) {
                    renderer.render.blend_mode(val0, val1);
                }
            },
            ESpineCommand::Graph(id, node) => {
                commands.entity(id.0).insert(GraphId(node));
            },
        }
    })
}

pub fn sys_spine_render_apply(
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
    });
    resource.verticeallocator.upload(&queue);
    resource.indicesallocator.upload(&queue);
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

pub struct ActionSpine;
impl ActionSpine {
    pub fn create_spine_renderer(
        id: KeySpineRenderer,
        rendersize: Option<(u32, u32)>,
        ctx: &mut SpineRenderContext,
        final_render_format: wgpu::TextureFormat,
    ) {
        ctx.create_renderer(id, rendersize.is_none());
        match rendersize {
            Some(rendersize) => {
                ctx.list.get_mut(&id).unwrap().render.target_format = wgpu::TextureFormat::Rgba8Unorm;
                ctx.list.get_mut(&id).unwrap().width = rendersize.0;
                ctx.list.get_mut(&id).unwrap().height = rendersize.1;
            },
            None => {
                ctx.list.get_mut(&id).unwrap().render.target_format = final_render_format;
            },
        }
    }

    pub fn spine_renderer_apply(
        id: KeySpineRenderer,
        name: Atom,
        to_screen: bool,
        render_graph: &mut PiRenderGraph,
    ) -> Result<NodeId, GraphError> {
        let key = String::from(name.as_str());
        match render_graph.add_node(key.clone(), SpineRenderNode::new(id), NodeId::null()) {
            Ok(v) => {
                // if to_screen {
                //     render_graph.add_depend(WindowRenderer::CLEAR_KEY, key.clone());
                //     render_graph.add_depend(key, WindowRenderer::KEY);
                // }
        
                // render_graph.dump_graphviz();
                Ok(v)
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    pub fn dispose_spine_renderer(
        id_renderer: KeySpineRenderer,
        ctx: &mut SpineRenderContext,
    ) {
        ctx.list.remove(&id_renderer);
    }

    pub fn spine_uniform(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        value: &[f32],
    ) {
        cmds.push(ESpineCommand::Uniform(id_renderer, value.to_vec()));
    }

    pub fn spine_shader(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        value: KeySpineShader,
    ) {
        cmds.push(ESpineCommand::Shader(id_renderer, Some(value)));
    }

    pub fn spine_use_texture(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        value: Handle<TextureRes>,
        sampler: Handle<SamplerRes>,
    ) {
        cmds.push(ESpineCommand::UseTexture(id_renderer, Some(value), Some(sampler)));
    }

    pub fn spine_draw(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        vertices: &[f32],
        indices: &[u16],
        vlen: u32,
        ilen: u32,
    ) {
        cmds.push(ESpineCommand::Draw(id_renderer, vertices.to_vec(), indices.to_vec(), vlen, ilen));
    }

    pub fn spine_texture(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        key: &str,
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
                view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb]
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
                    bytes_per_row: std::num::NonZeroU32::new(4 * width).map(|r| {r.get()}),
                    rows_per_image: std::num::NonZeroU32::new(height).map(|r| {r.get()}),
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
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            let textureres = TextureRes::new(width, height, (width * height * 4) as usize, texture_view, true, wgpu::TextureFormat::Rgba8UnormSrgb);
            
            if let Ok(texture) = asset_textures.insert(key_u64, textureres) {
                texture
            } else {
                return;
            }
        };

        let samplerdesc = SAMPLER_DESC.clone();

        let sampler = if let Some(sampler) = asset_samplers.get(&samplerdesc) {
            sampler
        } else {
            if let Ok(sampler) = asset_samplers.insert(samplerdesc.clone(), SamplerRes::new(&device, &samplerdesc)) {
                sampler
            } else {
                return;
            }
        };

        cmds.push(ESpineCommand::Texture(id_renderer, key_u64, texture, samplerdesc, sampler));
    }

    pub fn spine_reset(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
    ) {
        cmds.push(ESpineCommand::Reset(id_renderer));
    }

    pub fn spine_blend(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        value: bool,
    ) {
        cmds.push(ESpineCommand::Blend(id_renderer, value));
    }

    pub fn spine_blend_mode(
        cmds: &mut ActionListSpine,
        id_renderer: KeySpineRenderer,
        src: wgpu::BlendFactor,
        dst: wgpu::BlendFactor,
    ) {
        cmds.push(ESpineCommand::BlendMode(id_renderer, src, dst));
    }
}


#[derive(Resource, Default)]
pub struct SpineTextureLoad {
    pub success: Share<SegQueue<(Atom, Handle<TextureRes>)>>,
    pub fail: Share<SegQueue<(Atom, String)>>,
    pub list: Vec<Atom>,
}
impl SpineTextureLoad {
    pub fn load(&mut self, key: Atom) {
        self.list.push(key)
    }
}


fn sys_spine_texture_load(
    mut loader: ResMut<SpineTextureLoad>,
    device: Res<PiRenderDevice>,
    queue: Res<PiRenderQueue>,
    texture_assets_mgr: Res<ShareAssetMgr<TextureRes>>,
) {
    let mut list = replace(&mut loader.list, vec![]);
    list.drain(..).for_each(|k| {
        let result = AssetMgr::load(&texture_assets_mgr, &(k.asset_u64()));
        match result {
            LoadResult::Ok(r) => {
                loader.success.push((k, r));
            }
            ,
            _ => {
                let success = loader.success.clone();
                let fail = loader.fail.clone();
                let device = device.0.clone();
                let queue = queue.0.clone();
    
                RENDER_RUNTIME
                    .spawn(async move {
                        let desc = ImageTextureDesc {
                            url: &k,
                            device: &device,
                            queue: &queue,
                        };
        
                        let r = TextureRes::async_load(desc, result).await;
                        match r {
                            Ok(r) => {
                                success.push((k, r));
                            }
                            Err(e) => {
                                fail.push((k, format!("load image fail, {:?}", e)));
                            }
                        };
                    })
                    .unwrap();
            }
        }
    });
}


#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct SpineSystemSet;

#[derive(Clone, Resource)]
pub struct SpineAssetConfig {
    pub vertex_buffer: (usize, usize),
    pub bind_buffer: (usize, usize),
    pub bind_group: (usize, usize),
}
impl Default for SpineAssetConfig {
    fn default() -> Self {
        Self {
            vertex_buffer: (10 * 1024 * 1024, 60 * 1024),
            bind_buffer: (1 * 1024 * 1024, 60 * 1024),
            bind_group: (100 * 1024, 60 * 1024),
        }
    }
}

#[derive(Default)]
pub struct PluginSpineRenderer;
impl Plugin for PluginSpineRenderer {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<ShareAssetMgr<SamplerRes>>().is_none() {
            app.insert_resource(ShareAssetMgr::<SamplerRes>::new(GarbageEmpty(), false, 32 * 1024, 30 * 1000));
        }
        if app.world.get_resource::<ShareAssetMgr<TextureRes>>().is_none() {
            app.insert_resource(ShareAssetMgr::<TextureRes>::new(GarbageEmpty(), false, 32 * 1024 * 1024, 30 * 1000));
        }

        let cfg = if let Some(cfg) = app.world.get_resource::<SpineAssetConfig>() {
            cfg.clone()
        } else {
            app.world.insert_resource(SpineAssetConfig::default());
            app.world.get_resource::<SpineAssetConfig>().unwrap().clone()
        };
        
        let device = app.world.get_resource::<PiRenderDevice>().unwrap().0.clone();
        app.insert_resource(ActionListSpine::default())
            .insert_resource(SpineResource::new(&device, cfg.vertex_buffer.clone(), cfg.bind_buffer.clone(), cfg.bind_group.clone()))
            .insert_resource(SpineRenderContext::new())
            .insert_resource(SpineTextureLoad::default());

        app.add_systems(
			Update,
            (
                sys_spine_cmds,
                sys_spine_render_apply,
                sys_spine_texture_load
            ).chain().in_set(SpineSystemSet).before(PiRenderSystemSet)
        );

        app.add_system(apply_deferred.in_set(SpineSystemSet));

        // log::warn!("PluginSpineRenderer");
    }
}