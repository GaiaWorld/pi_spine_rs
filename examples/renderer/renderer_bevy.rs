use std::sync::Arc;

use bevy::{prelude::{App, Plugin, Startup}};
use image::GenericImageView;
use pi_atom::Atom;
use pi_bevy_asset::ShareAssetMgr;
use pi_bevy_ecs_extend::TShell;
use pi_bevy_render_plugin::{PiRenderPlugin, PiRenderGraph, PiRenderDevice, PiRenderQueue};
use pi_window_renderer::{PluginWindowRender, WindowRenderer};
use pi_render::{asset::TAssetKeyU64, rhi::{asset::TextureRes, sampler::{SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}}, renderer::sampler::SamplerRes};
use pi_scene_math::{Vector4, Matrix};
use pi_spine_rs::{PluginSpineRenderer, shaders::KeySpineShader, SpineRenderContext, ecs::{ResMut, Res, Commands}, ActionListSpine, KeySpineRenderer, ActionSpine};
use pi_async_rt::rt::AsyncRuntime;
use pi_hal::{init_load_cb, runtime::MULTI_MEDIA_RUNTIME, on_load};

use super::{vertices::VERTICES, indices::INDICES};

fn runner(
    mut commands: Commands,
    mut ctx: ResMut<SpineRenderContext>,
    mut render_graph: ResMut<PiRenderGraph>,
    device: Res<PiRenderDevice>,
    queue: Res<PiRenderQueue>,
    asset_textures: Res<ShareAssetMgr<TextureRes>>,
    asset_samplers: Res<ShareAssetMgr<SamplerRes>>,
    mut cmds: ResMut<ActionListSpine>,
    final_render: Res<WindowRenderer>,
) {
    let mut entitycmd = commands.spawn_empty();
    let id_renderer = KeySpineRenderer(entitycmd.id());
    ActionSpine::create_spine_renderer(id_renderer, None, &mut ctx, final_render.format());

    match ActionSpine::spine_renderer_apply(id_renderer, Atom::from("TestSpine"), true,  &mut render_graph) {
        Ok(nodeid) => { 
            entitycmd.insert(pi_bevy_render_plugin::component::GraphId(nodeid));

            //// Texture
            let diffuse_bytes = include_bytes!("../wanzhuqian.png");
            let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
            let diffuse_rgba = diffuse_image.as_bytes();
            let dimensions = diffuse_image.dimensions();
            
            let mut uniform_param = vec![];
            let matrix = Matrix::new(
                0.0016322123119607568 as f32,0.,0.,0.,
                0.,0.002176283160224557 as f32,0.,0.,
                0.,0.,-0.019999999552965164 as f32,0.,
                -0.05721032992005348 as f32,-0.950402557849884 as f32,-1.,1.
            );
            [
                0.0016322123119607568 as f32,0.,0.,0.,
                0.,0.002176283160224557 as f32,0.,0.,
                0.,0.,-0.019999999552965164 as f32,0.,
                -0.05721032992005348 as f32,-0.950402557849884 as f32,-1.,1.
            ].as_slice().iter().for_each(|v| {
                uniform_param.push(*v);
            });
            Vector4::new(0.5000, 0.5000, 0.5000, 0.).as_slice().iter().for_each(|v| {
                uniform_param.push(*v);
            });
            [1., 1., 1., 1.].iter().for_each(|v| {
                uniform_param.push(*v);
            });
            
            let samplerdesc = SamplerDesc {
                address_mode_u: EAddressMode::ClampToEdge,
                address_mode_v: EAddressMode::ClampToEdge,
                address_mode_w: EAddressMode::ClampToEdge,
                mag_filter: EFilterMode::Nearest,
                min_filter: EFilterMode::Nearest,
                mipmap_filter: EFilterMode::Nearest,
                compare: None,
                anisotropy_clamp: EAnisotropyClamp::None,
                border_color: None,
            };
            let sampler = if let Some(sampler) = asset_samplers.get(&samplerdesc) {
                sampler
            } else {
                if let Ok(sampler) = asset_samplers.insert(samplerdesc.clone(), SamplerRes::new(&device, &samplerdesc)) {
                    sampler
                } else {
                    return;
                }
            };
            if let Some(renderer) = ctx.get_mut(id_renderer) {
                // log::warn!("Cmd: Texture");
                renderer.render_mut().record_sampler(samplerdesc, sampler.clone());
            }
    
            let key_image = "../wanzhuqian.png";
            ActionSpine::spine_texture(&mut cmds, id_renderer, key_image.clone(), diffuse_rgba, dimensions.0, dimensions.1, &device, &queue, &asset_textures, &asset_samplers);
            ActionSpine::spine_shader(&mut cmds, id_renderer, KeySpineShader::TwoColoredTextured);
            ActionSpine::spine_uniform(&mut cmds, id_renderer, &uniform_param);
            ActionSpine::spine_use_texture(&mut cmds, id_renderer, asset_textures.get(&key_image.asset_u64()).unwrap(), sampler.clone());
            ActionSpine::spine_blend_mode(&mut cmds, id_renderer, wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha);
            ActionSpine::spine_draw(
                &mut cmds, 
                id_renderer,
                &VERTICES.as_slice()[0..9636],
                &INDICES.as_slice()[0..2352],
                9636,
                2352,
            );
            log::warn!("Init Ok!!!!!!");
        },
        Err(e) => {

        }
    }
}

pub struct Engine(App);
impl TShell for Engine {
    fn world(&self) -> &bevy::prelude::World {
        &self.0.world
    }

    fn world_mut(&mut self) -> &mut bevy::prelude::World {
        &mut self.0.world
    }

    fn app(&self) -> &App {
        &self.0
    }

    fn app_mut(&mut self) -> &mut App {
        &mut self.0
    }
}

pub struct PluginLocalLoad;
impl Plugin for PluginLocalLoad {
    fn build(&self, app: &mut App) {
        
        init_load_cb(Arc::new(|path: String| {
            MULTI_MEDIA_RUNTIME
                .spawn(async move {
                    log::debug!("Load {}", path);
                    let r = std::fs::read(path.clone()).unwrap();
                    on_load(&path, r);
                })
                .unwrap();
        }));
    }
}

pub fn run() -> Engine {
    let mut app = App::default();

	let mut window_plugin = bevy::window::WindowPlugin::default();
    if let Some(primary_window) = &mut window_plugin.primary_window {
        primary_window.resolution.set_physical_resolution(800, 600);
    }
	
	// app
		// .add_plugin(bevy::log::LogPlugin {
		// 	filter: "wgpu=info,pi_ui_render::components::user=debug".to_string(),
		// 	level: bevy::log::Level::INFO,
		// })
		app.add_plugins(bevy::input::InputPlugin::default());
		app.add_plugins(window_plugin);
        app.add_plugins(bevy::a11y::AccessibilityPlugin);
        app.add_plugins(bevy::winit::WinitPlugin::default());
		// .add_plugin(WorldInspectorPlugin::new())
        app.add_plugins(pi_bevy_asset::PiAssetPlugin::default());
		app.add_plugins(PiRenderPlugin::default());
		app.add_plugins(PluginLocalLoad);
		app.add_plugins(PluginWindowRender::default());
		app.add_plugins(PluginSpineRenderer::default());
        app.add_systems(Startup, runner);
        // ;
        
        app.world.get_resource_mut::<WindowRenderer>().unwrap().active = true;

    // let rendergraph = app.world.get_resource::<PiRenderGraph>().unwrap();
    // rendergraph.in
    let mut engine = Engine(app);

    engine.0.run();

    engine
}