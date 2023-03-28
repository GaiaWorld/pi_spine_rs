use bevy::{prelude::App, winit::WinitPlugin};
use image::GenericImageView;
use pi_atom::Atom;
use pi_bevy_asset::ShareAssetMgr;
use pi_bevy_ecs_extend::TShell;
use pi_bevy_render_plugin::{PiRenderPlugin, PiRenderGraph, PiRenderDevice, PiRenderQueue};
use pi_render::{asset::TAssetKeyU64, rhi::asset::TextureRes, renderer::sampler::SamplerRes};
use pi_scene_math::{Vector4, Matrix};
use pi_spine_rs::{PluginSpineRenderer, TInterfaceSpine, shaders::KeySpineShader, SpineRenderContext, ecs::{ResMut, Res}, SingleSpineCommands};

use super::{vertices::VERTICES, indices::INDICES};

fn runner(
    mut ctx: ResMut<SpineRenderContext>,
    mut render_graph: ResMut<PiRenderGraph>,
    device: Res<PiRenderDevice>,
    queue: Res<PiRenderQueue>,
    asset_textures: Res<ShareAssetMgr<TextureRes>>,
    asset_samplers: Res<ShareAssetMgr<SamplerRes>>,
    mut cmds: ResMut<SingleSpineCommands>,
) {
    let id_renderer = Engine::create_spine_renderer(Atom::from("Test"), None, &mut ctx, &mut render_graph);

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
    uniform_param.push(1.);

    let key_image = Atom::from("../wanzhuqian.png");
    Engine::spine_texture(id_renderer, key_image.clone(), diffuse_rgba, dimensions.0, dimensions.1, &device, &queue, &asset_textures, &asset_samplers, &mut ctx);
    Engine::spine_shader(id_renderer, KeySpineShader::TwoColoredTextured, &mut ctx);
    Engine::spine_uniform(id_renderer, &uniform_param, &mut ctx);
    Engine::spine_use_texture(id_renderer, key_image.asset_u64(), &mut ctx);
    Engine::spine_draw(
        id_renderer,
        &VERTICES.as_slice()[0..9636],
        
        &INDICES.as_slice()[0..2352],
        9636,
        2352,
        &mut ctx
    );
    log::warn!("Init Ok!!!!!!");
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
impl TInterfaceSpine for Engine {}

pub fn run() -> Engine {
    let mut app = App::default();

	let mut window_plugin = bevy::window::WindowPlugin::default();
	window_plugin.window.width = 800 as f32;
	window_plugin.window.height = 600 as f32;
	
	app
		// .add_plugin(bevy::log::LogPlugin {
		// 	filter: "wgpu=info,pi_ui_render::components::user=debug".to_string(),
		// 	level: bevy::log::Level::INFO,
		// })
		.add_plugin(bevy::input::InputPlugin::default())
		.add_plugin(window_plugin)
		.add_plugin(WinitPlugin::default())
		// .add_plugin(WorldInspectorPlugin::new())
		.add_plugin(PiRenderPlugin::default())
		.add_plugin(PluginSpineRenderer::default())
        .add_startup_system(runner)
        ;

    // let rendergraph = app.world.get_resource::<PiRenderGraph>().unwrap();
    // rendergraph.in
    let mut engine = Engine(app);

    engine.0.run();

    engine
}