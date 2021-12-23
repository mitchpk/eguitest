use {egui_miniquad as egui_mq, miniquad as mq};

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

struct Stage {
    egui_mq: egui_mq::EguiMq,
    pipeline: mq::Pipeline,
    bindings: mq::Bindings,
    offscreen_texture: mq::Texture,
    egui_demo_windows: egui_demo_lib::DemoWindows,
}

impl Stage {
    pub fn new(ctx: &mut mq::Context) -> Stage {
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -1., y: -1. }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  1., y: -1. }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  1., y:  1. }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -1., y:  1. }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = mq::Buffer::immutable(ctx, mq::BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = mq::Buffer::immutable(ctx, mq::BufferType::IndexBuffer, &indices);

        let bindings = mq::Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        let shader =
            mq::Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let pipeline = mq::Pipeline::new(
            ctx,
            &[mq::BufferLayout::default()],
            &[
                mq::VertexAttribute::new("pos", mq::VertexFormat::Float2),
                mq::VertexAttribute::new("uv", mq::VertexFormat::Float2),
            ],
            shader,
        );

        let (width, height) = ctx.screen_size();
        let offscreen_texture = mq::Texture::new_render_texture(
            ctx,
            mq::TextureParams {
                width: width as u32,
                height: height as u32,
                format: mq::TextureFormat::RGBA8,
                ..Default::default()
            },
        );

        let egui_mq = egui_mq::EguiMq::new(ctx);

        Stage {
            egui_mq,
            pipeline,
            bindings,
            offscreen_texture,
            egui_demo_windows: Default::default(),
        }
    }

    fn ui(&mut self) {
        let Self {
            egui_mq,
            egui_demo_windows,
            ..
        } = self;
        let egui_ctx = egui_mq.egui_ctx();
        egui::Window::new("egui ❤ miniquad").show(egui_ctx, |ui| {
            egui_demo_windows.ui(egui_ctx);
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            }
        });
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, ctx: &mut mq::Context) {
        self.egui_mq.begin_frame(ctx);
        self.ui();
        self.egui_mq.end_frame(ctx);

        let offscreen_pass = mq::RenderPass::new(ctx, self.offscreen_texture, None);

        self.egui_mq.set_render_pass(offscreen_pass);
        self.egui_mq.draw(ctx);

        ctx.begin_default_pass(mq::PassAction::clear_color(1.0, 1.0, 1.0, 1.));
        ctx.apply_pipeline(&self.pipeline);

        self.bindings.images = vec![self.offscreen_texture];

        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&shader::Uniforms {
            offset: (0.0, 0.0),
            resolution: ctx.screen_size()
        });
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn resize_event(&mut self, ctx: &mut mq::Context, width: f32, height: f32) {
        self.offscreen_texture.delete();
        self.offscreen_texture = mq::Texture::new_render_texture(
            ctx,
            mq::TextureParams {
                width: width as u32,
                height: height as u32,
                format: mq::TextureFormat::RGBA8,
                ..Default::default()
            },
        );
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(ctx, dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

fn main() {
    let conf = mq::conf::Conf {
        high_dpi: true,
        ..Default::default()
    };
    mq::start(conf, |mut ctx| {
        mq::UserData::owning(Stage::new(&mut ctx), ctx)
    });
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;
    uniform vec2 offset;
    varying lowp vec2 texcoord;
    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        texcoord = uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    uniform sampler2D tex;
    uniform lowp vec2 resolution;
    lowp float warp = 0.75;
    lowp float scan = 0.5;
    lowp float split = 1.0;
    void main() {
        lowp vec2 uv = texcoord;
        lowp vec2 dc = abs(0.5 - uv);
        dc *= dc;
        uv.x -= 0.5; uv.x *= 1.0+(dc.y*(0.3*warp)); uv.x += 0.5;
        uv.y -= 0.5; uv.y *= 1.0+(dc.x*(0.4*warp)); uv.y += 0.5;

        if (uv.y > 1.0 || uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0)
            gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        else {
            lowp float apply = abs(sin(uv.y * resolution.y / 2.0) * 0.5 * scan);
            lowp float r = texture2D(tex, vec2(uv.x + split / resolution.x, uv.y)).r;
            lowp float g = texture2D(tex, uv).g;
            lowp float b = texture2D(tex, vec2(uv.x - split / resolution.x, uv.y)).b;
            gl_FragColor = vec4(mix(vec3(r, g, b), vec3(0.0), apply), 1.0);
        }
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("offset", UniformType::Float2),
                    UniformDesc::new("resolution", UniformType::Float2),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
        pub resolution: (f32, f32),
    }
}
