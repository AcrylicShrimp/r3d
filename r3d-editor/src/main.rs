use pollster::FutureExt;
use r3d::{
    event::{event_types, EventHandler},
    gfx::{Camera, CameraClearMode, CameraPerspectiveProjectionAspect, CameraProjection, Color},
    math::Vec2,
    object::ObjectHandle,
    specs::{Builder, WorldExt},
    ui::{UIAnchor, UIElement, UIMargin, UIScaleMode, UIScaler, UISize},
    ContextHandle, Engine, EngineConfig, EngineExecError, EngineInitError, EngineLoopMode,
    EngineTargetFps,
};
use std::mem::MaybeUninit;
use thiserror::Error;

pub struct Application {
    pub main_camera: ObjectHandle,
    pub ui_camera: ObjectHandle,
    pub ui_root: ObjectHandle,
    pub ui_root_under: ObjectHandle,
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
    let main_camera_component = Camera::new(
        0x0000_FFFF,
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
    let ui_camera_component = Camera::new(
        0xFFFF_0000,
        1,
        CameraClearMode::depth_only(1.0, 0),
        CameraProjection::orthographic(10.0, -50.0, 50.0),
        &ctx.gfx_ctx().device,
        ctx.render_mgr_mut().bind_group_layout_cache(),
    );

    let mut object_mgr = ctx.object_mgr_mut();

    let mut world = ctx.world_mut();
    let (main_camera, builder) =
        object_mgr.create_object_builder(&mut world, Some("main-camera".to_owned()), None);
    builder.with(main_camera_component).build();

    let (ui_camera, builder) =
        object_mgr.create_object_builder(&mut world, Some("ui-camera".to_owned()), None);
    builder.with(ui_camera_component).build();

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

    let (ui_root_under, builder) =
        object_mgr.create_object_builder(&mut world, Some("ui-root-under".to_owned()), None);
    builder
        .with(UIElement {
            anchor: UIAnchor::new(Vec2::ZERO, Vec2::ONE * 0.5f32),
            margin: UIMargin::zero(),
            is_interactable: false,
        })
        .with(UISize {
            width: 0.0,
            height: 0.0,
        })
        .build();

    object_mgr
        .object_hierarchy_mut()
        .set_parent(ui_root_under.object_id, Some(ui_root.object_id));

    unsafe {
        APP = MaybeUninit::new(Application {
            main_camera,
            ui_camera,
            ui_root,
            ui_root_under,
        });
    }

    ctx.event_mgr()
        .add_handler(EventHandler::<event_types::Update>::new(|ctx, _| {
            update(ctx)
        }));
    ctx.event_mgr()
        .add_handler(EventHandler::<event_types::LateUpdate>::new(|ctx, _| {
            late_update(ctx)
        }));
}

fn update(ctx: &ContextHandle) {}

fn late_update(ctx: &ContextHandle) {
    // let world = ctx.world();
    // let sizes = world.read_component::<UISize>();
    // let ui_root_size = sizes.get(use_app().ui_root.entity).unwrap();
    // let ui_root_under_size = sizes.get(use_app().ui_root_under.entity).unwrap();

    // println!("root: {:?}", ui_root_size);
    // println!("root-under: {:?}", ui_root_under_size);
}
