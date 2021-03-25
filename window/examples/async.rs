use ::window::*;
use promise::spawn::spawn;
use std::any::Any;

struct MyWindow {
    allow_close: bool,
    cursor_pos: Point,
    render_pipeline: Option<wgpu::RenderPipeline>,
}

impl Drop for MyWindow {
    fn drop(&mut self) {
        eprintln!("MyWindow dropped");
    }
}

impl WindowCallbacks for MyWindow {
    fn can_close(&mut self) -> bool {
        true
    }

    fn destroy(&mut self) {
        eprintln!("destroy was called!");
        Connection::get().unwrap().terminate_message_loop();
    }

    fn resize(&mut self, dims: Dimensions, is_full_screen: bool) {
        eprintln!("resize {:?} is_full_screen={}", dims, is_full_screen);
    }

    fn key_event(&mut self, key: &KeyEvent, ctx: &dyn WindowOps) -> bool {
        eprintln!("{:?}", key);
        ctx.set_cursor(Some(MouseCursor::Text));
        false
    }

    fn mouse_event(&mut self, event: &MouseEvent, ctx: &dyn WindowOps) {
        self.cursor_pos = event.coords;
        ctx.invalidate();
        ctx.set_cursor(Some(MouseCursor::Arrow));

        if event.kind == MouseEventKind::Press(MousePress::Left) {
            eprintln!("{:?}", event);
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn created(&mut self, _window: &Window, gpu_context: &mut GpuContext) -> anyhow::Result<()> {
        log::info!("created gpu context");
        let shader = gpu_context
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "shader.wgsl"
                ))),
                flags: wgpu::ShaderFlags::all(),
            });

        let pipeline_layout =
            gpu_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let swapchain_format = gpu_context
            .adapter
            .get_swap_chain_preferred_format(&gpu_context.surface);

        self.render_pipeline
            .replace(
                gpu_context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[swapchain_format.into()],
                        }),
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                    }),
            );

        Ok(())
    }

    fn render(&mut self, frame: &wgpu::SwapChainTexture, gpu_context: &mut GpuContext) {
        let mut encoder = gpu_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 1.,
                            b: 0.,
                            a: 0.5,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(self.render_pipeline.as_ref().unwrap());
            rpass.draw(0..3, 0..1);
        }

        gpu_context.queue.submit(Some(encoder.finish()));
    }
}

async fn spawn_window() -> Result<(), Box<dyn std::error::Error>> {
    let win = Window::new_window(
        "myclass",
        "the title",
        800,
        600,
        Box::new(MyWindow {
            allow_close: false,
            cursor_pos: Point::new(100, 200),
            render_pipeline: None,
        }),
        None,
    )
    .await?;

    eprintln!("before show");
    win.show().await?;
    eprintln!("after show");
    win.apply(|myself, _win| {
        eprintln!("doing apply");
        if let Some(myself) = myself.downcast_ref::<MyWindow>() {
            eprintln!(
                "got myself; allow_close={}, cursor_pos:{:?}",
                myself.allow_close, myself.cursor_pos
            );
        }
        Ok(())
    })
    .await?;
    eprintln!("done with spawn_window");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut builder = pretty_env_logger::formatted_timed_builder();
    builder.filter(Some("window"), log::LevelFilter::Info);
    //builder.filter(None, log::LevelFilter::Info);
    builder.init();

    let conn = Connection::init()?;
    spawn(async {
        eprintln!("running this async block");
        match spawn_window().await {
            Ok(_) => eprintln!("Made a window!"),
            Err(err) => {
                eprintln!("{:#}", err);
                Connection::get().unwrap().terminate_message_loop();
            }
        }
        eprintln!("end of async block");
    })
    .detach();
    conn.run_message_loop()
}
