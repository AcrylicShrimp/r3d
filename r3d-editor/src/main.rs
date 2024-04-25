use assets::{FONT, MATERIAL_GLYPH, MATERIAL_SPRITE};
use pollster::FutureExt;
use r3d::{
    event::{event_types, EventHandler},
    fontdue::layout::{HorizontalAlign, VerticalAlign},
    gfx::{
        Camera, CameraClearMode, CameraPerspectiveProjectionAspect, CameraProjection, Color,
        NinePatch, NinePatchHandle, NinePatchTexelMapping, Texture, TextureHandle,
        UIElementRenderer, UIElementSprite, UITextRenderer,
    },
    math::{Quat, Vec2, Vec3},
    object::{Object, ObjectHandle},
    object_event::{object_event_types, ObjectEventHandler},
    specs::{Builder, WorldExt},
    transform::{Transform, TransformComponent},
    ui::{UIAnchor, UIElement, UIMargin, UIScaleMode, UIScaler, UISize},
    use_context,
    wgpu::TextureFormat,
    ContextHandle, Engine, EngineConfig, EngineExecError, EngineInitError, EngineLoopMode,
    EngineTargetFps,
};
use std::mem::MaybeUninit;
use thiserror::Error;

mod assets;

pub struct Application {
    pub camera: ObjectHandle,
    pub ui_root: ObjectHandle,
    pub ui_root_under: ObjectHandle,
    pub ui_text: ObjectHandle,
}

static mut APP: MaybeUninit<Application> = MaybeUninit::uninit();

pub fn use_app() -> &'static Application {
    unsafe { APP.assume_init_ref() }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("engine init error: {0}")]
    EngineInitError(#[from] EngineInitError),
    #[error("engine exec error: {0}")]
    EngineExecError(#[from] EngineExecError),
}

fn main() -> Result<(), Error> {
    let engine = Engine::new(EngineConfig {
        title: format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        resizable: true,
        width: 800,
        height: 600,
    })
    .block_on()?;

    init(engine.context());

    engine.run(EngineLoopMode::Poll, EngineTargetFps::VSync)?;
    Ok(())
}

fn init(ctx: ContextHandle) {
    let camera_component = Camera::new(
        0xFFFF_FFFF,
        0,
        CameraClearMode::All {
            color: Color::parse_hex("141414").unwrap(),
            depth: 1.0,
            stencil: 0,
        },
        CameraProjection::perspective(
            60.0,
            CameraPerspectiveProjectionAspect::Screen,
            0.01,
            1000.0,
        ),
        &ctx.gfx_ctx().device,
        ctx.render_mgr_mut().bind_group_layout_cache(),
    );

    let mut object_mgr = ctx.object_mgr_mut();

    let mut world = ctx.world_mut();
    let (camera, builder) =
        object_mgr.create_object_builder(&mut world, Some("camera".to_owned()), None);
    builder.with(camera_component).build();

    let (ui_root, builder) =
        object_mgr.create_object_builder(&mut world, Some("ui-root".to_owned()), None);
    builder
        .with(UIScaler {
            mode: UIScaleMode::Stretch,
            reference_size: Vec2::new(800.0, 600.0),
        })
        .with(UISize {
            width: 0.0,
            height: 0.0,
        })
        .build();

    let texture = TextureHandle::new(Texture::from_image(
        TextureFormat::Rgba8Unorm,
        &r3d::image::open("/Users/ashrimp/Sandbox/Rectangle 1.png")
            .unwrap()
            .flipv(),
        &ctx.gfx_ctx().device,
        &ctx.gfx_ctx().queue,
    ));
    let nine_patch = NinePatchHandle::new(NinePatch::new(
        texture.clone(),
        NinePatchTexelMapping::new(
            0,
            20,
            texture.width - 20,
            texture.width,
            0,
            20,
            texture.height - 20,
            texture.height,
        ),
    ));
    let mut ui_element_renderer = UIElementRenderer::new();
    ui_element_renderer.set_material(MATERIAL_SPRITE.clone());
    ui_element_renderer.set_sprite(
        UIElementSprite::nine_patch(nine_patch),
        &ctx.gfx_ctx().device,
        ctx.render_mgr_mut().bind_group_layout_cache(),
    );

    let (ui_root_under, builder) =
        object_mgr.create_object_builder(&mut world, Some("ui-root-under".to_owned()), None);
    builder
        .with(UIElement {
            anchor: UIAnchor::new(Vec2::ZERO, Vec2::ONE * 0.5f32),
            margin: UIMargin::zero(),
            is_interactable: true,
        })
        .with(UISize {
            width: 0.0,
            height: 0.0,
        })
        .with(ui_element_renderer)
        .build();

    object_mgr
        .object_hierarchy_mut()
        .set_parent(ui_root_under.object_id, Some(ui_root.object_id));

    let mut ui_text_renderer = UITextRenderer::new();
    ui_text_renderer.with_config(|config| {
        config.horizontal_align = HorizontalAlign::Center;
        config.vertical_align = VerticalAlign::Middle;
    });
    ui_text_renderer.set_color(Color::parse_hex("FFFFFF").unwrap());
    ui_text_renderer.set_font_size_with_recommended_values(36.0);
    ui_text_renderer.set_material(MATERIAL_GLYPH.clone());
    ui_text_renderer.set_font(FONT.clone());
    ui_text_renderer.set_text("iiiiWowVAAV\nHi!".to_owned());

    let (ui_text, builder) =
        object_mgr.create_object_builder(&mut world, Some("ui-text".to_owned()), None);
    builder
        .with(UIElement {
            anchor: UIAnchor::full(),
            margin: UIMargin::zero(),
            is_interactable: false,
        })
        .with(UISize {
            width: 0.0,
            height: 0.0,
        })
        .with(ui_text_renderer)
        .build();

    object_mgr
        .object_hierarchy_mut()
        .set_parent(ui_text.object_id, Some(ui_root_under.object_id));

    ctx.event_mgr()
        .add_handler(EventHandler::<event_types::Update>::new(|_| update()));
    ctx.event_mgr()
        .add_handler(EventHandler::<event_types::LateUpdate>::new(|_| {
            late_update()
        }));

    ctx.object_event_mgr().add_handler(
        ObjectEventHandler::<object_event_types::MouseEnterEvent>::new(
            Object::new(ui_root_under.entity, ui_root_under.object_id),
            |object, _| {
                on_mouse_enter(object);
            },
        ),
    );
    ctx.object_event_mgr().add_handler(
        ObjectEventHandler::<object_event_types::MouseLeaveEvent>::new(
            Object::new(ui_root_under.entity, ui_root_under.object_id),
            |object, _| {
                on_mouse_leave(object);
            },
        ),
    );

    unsafe {
        APP = MaybeUninit::new(Application {
            camera,
            ui_root,
            ui_root_under,
            ui_text,
        });
    }
}

fn update() {}

fn late_update() {
    // let world = use_context().world();
    // let sizes = world.read_component::<UISize>();
    // let ui_root_size = sizes.get(use_app().ui_root.entity).unwrap();
    // let ui_root_under_size = sizes.get(use_app().ui_root_under.entity).unwrap();
    // let ui_root_under_pos = use_app()
    //     .ui_root_under
    //     .component::<TransformComponent>()
    //     .world_position();
    // let ui_text_pos = use_app()
    //     .ui_text
    //     .component::<TransformComponent>()
    //     .world_position();
    // let ui_text_size = sizes.get(use_app().ui_text.entity).unwrap();

    // println!("ui-root-under-pos: {:?}", ui_root_under_pos);
    // println!("ui-text-pos: {:?}", ui_text_pos);

    // println!("root: {:?}", ui_root_size);
    // println!("root-under: {:?}", ui_root_under_size);
    // println!("text: {:?}", ui_text_size);
}

fn on_mouse_enter(object: Object) {
    println!("on_mouse_enter: {:?}", object);
}

fn on_mouse_leave(object: Object) {
    println!("on_mouse_leave: {:?}", object);
}
