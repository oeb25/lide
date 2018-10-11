#![feature(transpose_result)]

extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate mg;
extern crate warmy;

use glutin::GlContext;

fn rip<T, C>(e: warmy::load::StoreErrorOr<T, C>) -> T::Error
where
    T: warmy::Load<C> + std::fmt::Debug,
{
    match e {
        warmy::load::StoreErrorOr::ResError(e) => e,
        e => panic!("{:?}", e),
    }
}

#[derive(Debug)]
struct FromFS {
    src: String,
    deps: Vec<warmy::DepKey>,
}

impl<C> warmy::Load<C> for FromFS {
    type Key = warmy::FSKey;

    type Error = std::io::Error;

    fn load(
        key: Self::Key,
        storage: &mut warmy::Storage<C>,
        ctx: &mut C,
    ) -> Result<warmy::Loaded<Self>, Self::Error> {
        println!("loading {:?}", key);

        let path = key.as_path();

        let mut deps = vec![];

        let src: Result<_, std::io::Error> = std::fs::read_to_string(path)?
            .lines()
            .map(|x| {
                let y = x.trim_left();
                let res = if y.starts_with("#include") {
                    let offset = "#include \"".len();
                    let end = y.len() - 1;
                    let req = &y[offset..end];

                    let path = path.strip_prefix(storage.root()).unwrap();

                    let req = path.parent().unwrap().to_str().unwrap().to_owned() + "/" + req;
                    let key = warmy::FSKey::new(req);
                    let res = storage.get(&key, ctx).map_err(rip)?;
                    deps.push(key.into());
                    let r: &FromFS = &res.borrow();
                    for d in r.deps.iter() {
                        if !deps.contains(d) {
                            deps.push(d.clone());
                        }
                    }
                    r.src.to_owned()
                } else {
                    x.to_owned()
                };
                Ok(res + "\n")
            })
            .collect();

        let from_fs = FromFS {
            src: src?,
            deps: deps.clone(),
        };

        Ok(warmy::load::Loaded::with_deps(from_fs.into(), deps))
    }
}

#[derive(Clone, Hash)]
struct ShaderSrc<'a> {
    vert: &'a str,
    geom: Option<&'a str>,
    frag: &'a str,
}

#[derive(Debug)]
struct MyShader {
    vert: String,
    frag: String,
    program: mg::Program,
    deps: Vec<warmy::DepKey>,
}

impl<'a> Into<warmy::key::LogicalKey> for ShaderSrc<'a> {
    fn into(self) -> warmy::key::LogicalKey {
        let mut p = self.vert.to_owned() + "," + self.frag;
        match self.geom {
            Some(q) => {
                p += ",";
                p += q;
            }
            None => {}
        }
        warmy::key::LogicalKey::new(p)
    }
}

impl<C> warmy::Load<C> for MyShader {
    type Key = warmy::key::LogicalKey;

    type Error = std::io::Error;

    fn load(
        key: Self::Key,
        storage: &mut warmy::Storage<C>,
        ctx: &mut C,
    ) -> Result<warmy::Loaded<Self>, Self::Error> {
        println!("loading {:?}", key);

        let mut deps = key.as_str().split(",");
        let vert = deps.next().unwrap();
        let frag = deps.next().unwrap();
        let geom = deps.next();

        let vert_key = warmy::FSKey::new(vert);
        let frag_key = warmy::FSKey::new(frag);
        let geom_key = geom.map(|geom| warmy::FSKey::new(geom));

        let vert_src: warmy::Res<FromFS> = storage.get(&vert_key, ctx).map_err(rip)?;
        let frag_src: warmy::Res<FromFS> = storage.get(&frag_key, ctx).map_err(rip)?;
        let geom_src: Option<warmy::Res<FromFS>> = geom_key
            .map(|geom_key| storage.get(&geom_key, ctx))
            .transpose()
            .map_err(rip)?;
        let vert = &vert_src.borrow();
        let frag = &frag_src.borrow();
        let geom = geom_src.as_ref().map(|geom_src| geom_src.borrow());
        let mut deps = vec![vert_key.into(), frag_key.into()];
        for d in vert.deps.iter().chain(frag.deps.iter()) {
            if !deps.contains(d) {
                deps.push(d.clone());
            }
        }
        if let Some(geom) = &geom {
            for d in geom.deps.iter() {
                if !deps.contains(d) {
                    deps.push(d.clone());
                }
            }
        }

        let vert_src = "#version 330\n".to_string() + &vert.src + "\n";
        let frag_src = "#version 330\n".to_string() + &frag.src + "\n";
        let geom_src = geom.map(|geom| "#version 330\n".to_string() + &geom.src + "\n");

        let program = mg::Program::new_from_src(
            &vert_src,
            match &geom_src {
                Some(x) => Some(x),
                None => None,
            },
            &frag_src,
        ).expect("unable to create program");

        let res = MyShader {
            vert: vert_src,
            frag: frag_src,
            program,
            deps: deps.clone(),
        };

        Ok(warmy::Loaded::with_deps(res.into(), deps))
    }
}

struct RenderTarget {
    tex: mg::Texture,
    fbo: mg::Framebuffer,
}

impl RenderTarget {
    fn new(w: u32, h: u32) -> RenderTarget {
        use mg::FramebufferBinderDrawer;

        let mut fbo = mg::Framebuffer::new();
        let mut rbo = mg::Renderbuffer::new();
        let tex = mg::Texture::new(mg::TextureKind::Texture2d);

        tex.bind()
            .empty(
                mg::TextureTarget::Texture2d,
                0,
                mg::TextureInternalFormat::Rgb,
                w,
                h,
                mg::TextureFormat::Rgb,
                mg::GlType::UnsignedByte,
            )
            .parameter_int(mg::TextureParameter::MinFilter, gl::LINEAR as i32)
            .parameter_int(mg::TextureParameter::MagFilter, gl::LINEAR as i32);

        rbo.bind()
            .storage(mg::TextureInternalFormat::DepthComponent, w, h);

        fbo.bind()
            .texture_2d(
                mg::Attachment::Color0,
                mg::TextureTarget::Texture2d,
                &tex,
                0,
            )
            .renderbuffer(mg::Attachment::Depth, &rbo)
            .draw_buffers(&[mg::Attachment::Color0]);

        RenderTarget { fbo, tex }
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello world!")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_gl_profile(glutin::GlProfile::Core);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window
            .context()
            .make_current()
            .expect("failed to make current");
    }
    gl::load_with(|s| gl_window.context().get_proc_address(s) as *const _);

    use warmy::{Store, StoreOpt};

    let ctx = &mut ();
    let mut store = Store::new(StoreOpt::default())?;
    let water_shader: warmy::res::Res<MyShader> = {
        let src = ShaderSrc {
            vert: "/assets/shaders/rect.vert",
            geom: Some("/assets/shaders/rect.geom"),
            frag: "/assets/shaders/scene.frag",
        };
        let src: warmy::LogicalKey = src.into();
        store.get(&src, ctx)?
    };
    let screen_shader: warmy::res::Res<MyShader> = {
        let src = ShaderSrc {
            vert: "/assets/shaders/rect.vert",
            geom: Some("/assets/shaders/rect.geom"),
            frag: "/assets/shaders/screen.frag",
        };
        let src: warmy::LogicalKey = src.into();
        store.get(&src, ctx)?
    };

    println!("{:?}", water_shader.borrow());

    let (mut w, mut h) = gl_window.window().get_inner_size().unwrap();
    let hidpi_factor = gl_window.window().hidpi_factor();

    let mut render_target = RenderTarget::new(w, h);
    let mut display_fbo = unsafe { mg::Framebuffer::window() };

    let mut rect_vao = {
        let mut rect_vao = mg::VertexArray::new();
        let mut rect_vbo = mg::VertexBuffer::from_data(&[[0.0, 0.0, 0.0f32]]);
        rect_vao.bind().vbo_attrib(&rect_vbo.bind(), 0, 1, 0);
        rect_vao
    };

    let mut t = 0.0;
    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => {
                    running = false;
                }
                glutin::WindowEvent::Resized(w_, h_) => {
                    // println!("{:?}", ((w, h), (w_, h_)));
                    w = (w_ as f32 / hidpi_factor) as u32;
                    // w = w_;
                    h = (h_ as f32 / hidpi_factor) as u32;
                    // h = h_;
                    render_target = RenderTarget::new(w, h);
                    gl_window.resize(w_, h_)
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => match input {
                    glutin::KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_keycode),
                        ..
                    } => match virtual_keycode {
                        glutin::VirtualKeyCode::Escape => {
                            if state == glutin::ElementState::Pressed {
                                running = false;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        });

        t += 1.0;

        unsafe {
            gl::Viewport(0, 0, w as i32, h as i32);
        }

        {
            let mut shader = water_shader.borrow_mut();
            let program = shader.program.bind();
            program.bind_float("time", t);
            rect_vao.bind().draw_arrays(
                &render_target.fbo.bind(),
                &program,
                mg::DrawMode::Points,
                0,
                1,
            );
        }

        unsafe {
            gl::Viewport(
                0,
                0,
                (w as f32 * hidpi_factor) as i32,
                (h as f32 * hidpi_factor) as i32,
            );
        }

        {
            let mut shader = screen_shader.borrow_mut();
            let program = shader.program.bind();
            program.bind_float("time", t).bind_texture(
                "scene",
                &render_target.tex,
            );
            rect_vao
                .bind()
                .draw_arrays(&display_fbo.bind(), &program, mg::DrawMode::Points, 0, 1);
        }

        store.sync(ctx);
        gl_window.swap_buffers()?;
    }

    Ok(())
}
