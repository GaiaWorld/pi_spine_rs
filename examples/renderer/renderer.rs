
use std::{num::NonZeroU32, time::SystemTime, sync::Arc};

use image::{GenericImageView};
use pi_assets::{mgr::AssetMgr, asset::GarbageEmpty};
use pi_atom::Atom;
use pi_render::{components::view::target_alloc::{SafeAtlasAllocator, ShareTargetView}, rhi::{device::RenderDevice, asset::{RenderRes, TextureRes, }, bind_group::BindGroup, RenderQueue, sampler::{Sampler, SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}, }, renderer::{draw_obj_list::DrawList, vertex_buffer::VertexBufferAllocator}, depend_graph::graph::DependGraph};
use pi_scene_math::{Matrix, Vector4};
use pi_share::Share;
use pi_spine_render::{renderer::Renderer, vertex_buffer::SpineVertexBufferAllocator, shaders::{SingleSpinePipelinePool, SingleSpineBindGroupLayout, KeySpineShader}, binds::param::BindBufferAllocator};
use winit::{window::Window, event::WindowEvent};

use super::{indices::INDICES, vertices::VERTICES};

fn test_texture(device: &wgpu::Device, asset_mgr_texture: &Share<AssetMgr<TextureRes>>) -> wgpu::Texture {
    let texture_size = wgpu::Extent3d {
        width: 2048, // dimensions.0,
        height: 2048, //dimensions.1,
        depth_or_array_layers: 1,
    };
    let diffuse_texture = device.create_texture(
        &wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB so we need to reflect that here.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::COPY_DST,
            label: None,
        }
    );
    // let texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    // texture_view
    diffuse_texture
}

pub struct State {
    // texture_view: wgpu::Texture,
    pub surface: wgpu::Surface,
    pub renderdevice: RenderDevice,
    pub queue: RenderQueue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub value_test: u8,
    pub diffuse_size: wgpu::Extent3d,
    // pub diffuse_buffer: wgpu::Buffer,
    pub lasttime: SystemTime,
    pub asset_mgr_bindgroup: Share<AssetMgr<RenderRes<BindGroup>>>,
    pub asset_mgr_texture: Share<AssetMgr<TextureRes>>,
    pub asset_mgr_sampler: Share<AssetMgr<Sampler>>,
    pub vb_allocator: VertexBufferAllocator,
    pub pipelines: SingleSpinePipelinePool,
    pub bind_group_layouts: SingleSpineBindGroupLayout,
    bind_buffers: BindBufferAllocator,
    renderer: Renderer,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        )
        .await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::DEPTH_CLIP_CONTROL,
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None
        )
        .await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter).get(0).unwrap().clone(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        ///// 

        //// Texture
        let diffuse_bytes = include_bytes!("../wanzhuqian.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.as_bytes();
        let dimensions = diffuse_image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                label: Some("diffuse_texture"),
            }
        );
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );
        let texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let asset_mgr_bindgroup: Share<AssetMgr<RenderRes<BindGroup>>> = AssetMgr::<RenderRes::<BindGroup>>::new(GarbageEmpty(), false, 1 * 1024 * 1024 , 100 * 1000);
        let asset_mgr_texture: Share<AssetMgr<TextureRes>> = AssetMgr::<TextureRes>::new(GarbageEmpty(), false,  1024 * 1024 , 100 * 1000);
        let asset_mgr_sampler: Share<AssetMgr<Sampler>> = AssetMgr::<Sampler>::new(GarbageEmpty(), false, 10 * 1024 * 1024 , 100 * 1000);

        asset_mgr_texture.insert(Atom::from("wanzhuqian.png").get_hash() as u64, TextureRes::new(texture_size.width, texture_size.height, (texture_size.width * texture_size.height * 4) as usize, texture_view, true, wgpu::TextureFormat::Rgba8UnormSrgb));

        // let texture_view = test_texture(&device, &asset_mgr_texture);
        // for i in 0..1 {
        //     let texture_view = test_texture(&device, &asset_mgr_texture);
            
        //     asset_mgr_texture.insert(i as u64, TextureRes::new(texture_size.width, texture_size.height, (texture_size.width * texture_size.height * 4) as usize, texture_view, true));
        // }

        let renderdevice = RenderDevice::from(Arc::new(device));
        let queue = RenderQueue::from(Arc::new(queue));

        let vb_allocator = VertexBufferAllocator::new(64 * 1024, 60 * 1000);
        let pipelines = SingleSpinePipelinePool::new(&renderdevice);
        let bind_group_layouts = SingleSpineBindGroupLayout::new(&renderdevice);
        let bind_buffers = BindBufferAllocator::new();


        let desc = SamplerDesc {
            address_mode_u: EAddressMode::ClampToEdge,
            address_mode_v: EAddressMode::ClampToEdge,
            address_mode_w: EAddressMode::ClampToEdge,
            mag_filter: EFilterMode::Linear,
            min_filter: EFilterMode::Linear,
            mipmap_filter: EFilterMode::Nearest,
            compare: None,
            anisotropy_clamp: EAnisotropyClamp::None,
            border_color: None,
        };
        let sampler = Sampler::new(&renderdevice, &desc);

        asset_mgr_sampler.insert(desc, sampler);
        
        let mut renderer = Renderer::new();

        Self {
            // texture_view,
            surface,
            renderdevice,
            queue,
            config,
            size,
            value_test: 0,
            diffuse_size: texture_size,
            lasttime: std::time::SystemTime::now(),
            asset_mgr_bindgroup,
            asset_mgr_texture,
            asset_mgr_sampler,
            vb_allocator,
            pipelines,
            bind_group_layouts,
            bind_buffers,
            renderer,
        }
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.renderdevice.wgpu_device(), &self.config);
        }
    }

    pub fn input(
        &mut self,
        event: &WindowEvent,
    ) -> bool {
        false
    }

    pub fn update(
        &mut self,
    ) {
        let mut r = self.value_test;
        if r == 255 {
            r = 0;
        } else {
            r = r + 1;
        }
        self.value_test = r;
    }

    pub fn render(
        &mut self,
    ) -> Result<(), wgpu::SurfaceError> {
        let last_time = SystemTime::now();
        let output = self.surface.get_current_texture()?;

        // BGRASrgb
        let ouput_format = self.config.format;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor {
                label: None,
                format: Some(ouput_format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count:  NonZeroU32::new(0),
                base_array_layer: 0,
                array_layer_count: None,
            }
        );

        let mut encoder = self.renderdevice.wgpu_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Ender Encoder")
            }
        );

        self.clear(&mut encoder, &view);
        
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let receive_w = self.size.width - 200 as u32;
        let receive_h = self.size.height - 200 as u32;
        let receive_width = self.size.width;
        let receive_height = self.size.height;

        
        let renderer = &mut self.renderer;
        renderer.reset();
        {
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
            
            
            renderer.uniform(
                &uniform_param,
                &mut self.bind_buffers,
                &self.renderdevice,
                &self.queue,
            );
            renderer.blend(true);
            renderer.blend_mode(
                wgpu::BlendFactor::SrcAlpha,
                wgpu::BlendFactor::OneMinusSrcAlpha,
            );

            println!("{:?}", Matrix::identity());
            println!("{:?}", matrix);

            // renderer.shader(Some(KeySpineShader::Colored));
            // renderer.draw(
            //     None,
            //     None,&[
            //         -0.5 * 800., -0.5 * 800.,  1.0, 0., 0., 1.0,
            //         -0.5 * 800.,  0.5 * 800.,  1.0, 0., 0., 1.0,
            //          0.5 * 800.,  0.5 * 800.,  0.0, 0., 0., 1.0,
            //          0.5 * 800., -0.5 * 800.,  1.0, 0., 0., 1.0,
            //     ],
            //     Some(
            //         &[
            //             0, 1, 2,
            //             0, 2, 3
            //         ]
            //     ),
            //     24,
            //     6,
            //     &self.renderdevice,
            //     &self.queue,
            //     ouput_format,
            //     &self.asset_mgr_bindgroup,
            //     &mut self.vb_allocator,
            //     &mut self.pipelines,
            //     &mut self.bind_group_layouts
            // );

            let texture = self.asset_mgr_texture.get(&(Atom::from("wanzhuqian.png").get_hash() as u64));
            let sampler = self.asset_mgr_sampler.get(&SamplerDesc {
                address_mode_u: EAddressMode::ClampToEdge,
                address_mode_v: EAddressMode::ClampToEdge,
                address_mode_w: EAddressMode::ClampToEdge,
                mag_filter: EFilterMode::Linear,
                min_filter: EFilterMode::Linear,
                mipmap_filter: EFilterMode::Nearest,
                compare: None,
                anisotropy_clamp: EAnisotropyClamp::None,
                border_color: None,
            });
            renderer.blend(true);
            renderer.blend_mode(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha);
            // println!("{:?}, {:?}", texture.is_some(), sampler.is_some());
            renderer.shader(Some(KeySpineShader::TwoColoredTextured));
            renderer.draw(
                texture,
                sampler,
                &VERTICES.as_slice()[0..9636],
                Some(
                    &INDICES.as_slice()[0..2352]
                ),
                9636,
                2352,
                &self.renderdevice,
                &self.queue,
                ouput_format,
                &self.asset_mgr_bindgroup,
                &mut self.vb_allocator,
                &mut self.pipelines,
                &mut self.bind_group_layouts
            );
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
            
            renderpass.set_viewport(
                0 as f32,
                0 as f32,
                800 as f32,
                600 as f32,
                0.,
                1.
            );
            DrawList::render(
                &renderer.drawlist().list,
                &mut renderpass
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        // let new_time = SystemTime::now();
        // println!("{:?}", new_time.duration_since(last_time));
        Ok(())
    }

    fn clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView
    ) {
        let renderpass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(
                        wgpu::RenderPassColorAttachment {
                            view: view,
                            resolve_target: None,
                            ops:wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: self.value_test as f64 / 255.0, 
                                        g: 0.21, 
                                        b: 0.41, 
                                        a: 1.0, 
                                    }
                                ),
                                store: true
                            }
                        }
                    )
                ],
                depth_stencil_attachment: None,
            }
        );
    }
}
