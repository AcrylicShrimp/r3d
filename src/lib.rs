use self::{
    ecs_system::{
        render::RenderSystem, update_camera_transform_buffer::UpdateCameraTransformBufferSystem,
    },
    gfx::{
        Camera, DepthStencilMode, GfxContext, GfxContextCreationError, GfxContextHandle,
        RenderManager, ScreenManager, ShaderManager,
    },
    time::TimeManager,
    vsync::TargetFrameInterval,
};
use codegen::Handle;
use ecs_system::{
    make_ui_scaler_dirty::MakeUIScalerDirty, update_ui_element::UpdateUIElement,
    update_ui_raycast_grid::UpdateUIRaycastGrid, update_ui_scaler::UpdateUIScaler,
};
use event::{event_types, EventManager};
use gfx::{BuiltInShaderManager, GlyphManager, MeshRenderer, UIElementRenderer, UITextRenderer};
use input::InputManager;
use math::Vec2;
use object::{Object, ObjectManager};
use object_event::ObjectEventManager;
use specs::prelude::*;
use std::{
    cell::{Ref, RefCell, RefMut},
    mem::MaybeUninit,
    num::NonZeroU32,
    time::Instant,
};
use thiserror::Error;
use transform::Transform;
use ui::{UIElement, UIEventManager, UIRaycastManager, UIScaler, UISize};
use wgpu::MaintainBase;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod asset;
pub mod ecs_system;
pub mod event;
pub mod gfx;
pub mod input;
pub mod math;
pub mod object;
pub mod object_event;
pub mod time;
pub mod transform;
pub mod ui;
pub mod util;
pub mod vsync;

// re-exports.
pub use fontdue;
pub use image;
pub use russimp;
pub use specs;
pub use wgpu;

static mut CONTEXT: MaybeUninit<ContextHandle> = MaybeUninit::uninit();

pub fn use_context() -> &'static ContextHandle {
    unsafe { CONTEXT.assume_init_ref() }
}

// TODO: If we borrow any of the context's fields more than once, it will panic.
// I think it is a big problem because it's very hard to ensure that any of function will not borrow
// the context's fields more than once in their call stack.
// Wrapping fields with RefCell is not a good solution I think; it groups too much fields into one lock.
// How about to make managers smaller?
#[derive(Handle)]
pub struct Context {
    window: Window,
    gfx_ctx: GfxContextHandle,
    world: RefCell<World>,
    object_mgr: RefCell<ObjectManager>,
    screen_mgr: RefCell<ScreenManager>,
    render_mgr: RefCell<RenderManager>,
    glyph_mgr: RefCell<GlyphManager>,
    shader_mgr: ShaderManager,
    built_in_shader_mgr: BuiltInShaderManager,
    ui_raycast_mgr: RefCell<UIRaycastManager>,
    ui_event_mgr: RefCell<UIEventManager>,
    time_mgr: RefCell<TimeManager>,
    input_mgr: RefCell<InputManager>,
    event_mgr: EventManager,
    object_event_mgr: ObjectEventManager,
}

impl Context {
    pub fn new(window: Window, gfx_ctx: GfxContext, screen_width: u32, screen_height: u32) -> Self {
        let gfx_ctx = GfxContextHandle::new(gfx_ctx);
        let world = World::new().into();
        let object_mgr = ObjectManager::new().into();
        let screen_mgr = ScreenManager::new(screen_width, screen_height).into();
        let render_mgr: RefCell<RenderManager> = RenderManager::new(
            gfx_ctx.clone(),
            PhysicalSize::new(screen_width, screen_height),
            DepthStencilMode::DepthOnly,
        )
        .into();
        let glyph_mgr = GlyphManager::new(gfx_ctx.clone()).into();
        let shader_mgr = ShaderManager::new(gfx_ctx.clone());
        let mut built_in_shader_mgr = BuiltInShaderManager::new();
        built_in_shader_mgr.init(
            &shader_mgr,
            render_mgr.borrow_mut().bind_group_layout_cache(),
        );
        let ui_raycast_mgr = UIRaycastManager::new().into();
        let ui_event_mgr = UIEventManager::new().into();
        let time_mgr = TimeManager::new().into();
        let input_mgr = InputManager::new().into();
        let event_mgr = EventManager::new();
        let object_event_mgr = ObjectEventManager::new();

        Self {
            window,
            gfx_ctx,
            world,
            object_mgr,
            screen_mgr,
            render_mgr,
            glyph_mgr,
            shader_mgr,
            built_in_shader_mgr: built_in_shader_mgr.into(),
            ui_raycast_mgr,
            ui_event_mgr,
            time_mgr,
            input_mgr,
            event_mgr,
            object_event_mgr,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn gfx_ctx(&self) -> &GfxContextHandle {
        &self.gfx_ctx
    }

    pub fn world(&self) -> Ref<World> {
        self.world.borrow()
    }

    pub fn world_mut(&self) -> RefMut<World> {
        self.world.borrow_mut()
    }

    pub fn object_mgr(&self) -> Ref<ObjectManager> {
        self.object_mgr.borrow()
    }

    pub fn object_mgr_mut(&self) -> RefMut<ObjectManager> {
        self.object_mgr.borrow_mut()
    }

    pub fn screen_mgr(&self) -> Ref<ScreenManager> {
        self.screen_mgr.borrow()
    }

    pub fn screen_mgr_mut(&self) -> RefMut<ScreenManager> {
        self.screen_mgr.borrow_mut()
    }

    pub fn render_mgr(&self) -> Ref<RenderManager> {
        self.render_mgr.borrow()
    }

    pub fn render_mgr_mut(&self) -> RefMut<RenderManager> {
        self.render_mgr.borrow_mut()
    }

    pub fn glyph_mgr(&self) -> Ref<GlyphManager> {
        self.glyph_mgr.borrow()
    }

    pub fn glyph_mgr_mut(&self) -> RefMut<GlyphManager> {
        self.glyph_mgr.borrow_mut()
    }

    pub fn shader_mgr(&self) -> &ShaderManager {
        &self.shader_mgr
    }

    pub fn built_in_shader_mgr(&self) -> &BuiltInShaderManager {
        &self.built_in_shader_mgr
    }

    pub fn ui_raycast_mgr(&self) -> Ref<UIRaycastManager> {
        self.ui_raycast_mgr.borrow()
    }

    pub fn ui_raycast_mgr_mut(&self) -> RefMut<UIRaycastManager> {
        self.ui_raycast_mgr.borrow_mut()
    }

    pub fn ui_event_mgr(&self) -> Ref<UIEventManager> {
        self.ui_event_mgr.borrow()
    }

    pub fn ui_event_mgr_mut(&self) -> RefMut<UIEventManager> {
        self.ui_event_mgr.borrow_mut()
    }

    pub fn time_mgr(&self) -> Ref<TimeManager> {
        self.time_mgr.borrow()
    }

    pub fn time_mgr_mut(&self) -> RefMut<TimeManager> {
        self.time_mgr.borrow_mut()
    }

    pub fn input_mgr(&self) -> Ref<InputManager> {
        self.input_mgr.borrow()
    }

    pub fn input_mgr_mut(&self) -> RefMut<InputManager> {
        self.input_mgr.borrow_mut()
    }

    pub fn event_mgr(&self) -> &EventManager {
        &self.event_mgr
    }

    pub fn object_event_mgr(&self) -> &ObjectEventManager {
        &self.object_event_mgr
    }
}

pub struct Engine {
    event_loop: EventLoop<()>,
    ctx: ContextHandle,
}

impl Engine {
    pub async fn new(config: EngineConfig) -> Result<Self, EngineInitError> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_visible(false)
            .with_title(config.title)
            .with_resizable(config.resizable)
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .build(&event_loop)
            .unwrap();
        let gfx_ctx = GfxContext::new(&window).await?;
        let ctx = ContextHandle::new(Context::new(window, gfx_ctx, config.width, config.height));

        unsafe {
            CONTEXT.write(ctx.clone());
        }

        {
            let mut world = ctx.world_mut();
            world.register::<Object>();
            world.register::<Transform>();

            world.register::<Camera>();
            world.register::<MeshRenderer>();
            world.register::<UIElementRenderer>();
            world.register::<UITextRenderer>();

            world.register::<UISize>();
            world.register::<UIScaler>();
            world.register::<UIElement>();
        }

        {
            let scale_factor = ctx.window.scale_factor();
            let physical_size =
                LogicalSize::new(config.width, config.height).to_physical(scale_factor);
            let mut screen_mgr = ctx.screen_mgr_mut();
            screen_mgr.update_scale_factor(scale_factor, physical_size);
            ctx.gfx_ctx().resize(physical_size);
        }

        Ok(Self { event_loop, ctx })
    }

    pub fn context(&self) -> ContextHandle {
        self.ctx.clone()
    }

    pub fn run(
        self,
        loop_mode: EngineLoopMode,
        target_fps: EngineTargetFps,
    ) -> Result<(), EngineExecError> {
        let mut make_ui_scaler_dirty = MakeUIScalerDirty::new(self.ctx.clone());
        let mut update_ui_scaler = UpdateUIScaler::new(self.ctx.clone());
        let mut update_ui_element = UpdateUIElement::new(self.ctx.clone());
        let mut update_ui_raycast_grid = UpdateUIRaycastGrid::new(self.ctx.clone());
        let mut update_camera_transform_buffer_system =
            UpdateCameraTransformBufferSystem::new(self.ctx.clone());
        let mut render_system = RenderSystem::new(
            &self.ctx.gfx_ctx.device,
            self.ctx.render_mgr_mut().bind_group_layout_cache(),
        );

        self.ctx.window.set_visible(true);

        let window_id = self.ctx.window.id();
        let mut window_occluded = false;
        let mut target_frame_interval = TargetFrameInterval::new(
            match target_fps {
                EngineTargetFps::VSync => None,
                EngineTargetFps::MilliHertz(millihertz) => Some(millihertz),
                EngineTargetFps::Unlimited => None,
            },
            self.ctx.window(),
        );
        let mut last_frame_time = Instant::now();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = match loop_mode {
                EngineLoopMode::Wait => ControlFlow::Wait,
                EngineLoopMode::Poll => ControlFlow::Poll,
            };

            match event {
                Event::MainEventsCleared => {
                    if loop_mode == EngineLoopMode::Wait {
                        return;
                    }

                    let now = Instant::now();

                    if now - last_frame_time < target_frame_interval.interval() {
                        return;
                    }

                    last_frame_time = now;

                    {
                        let mut time_mgr = self.ctx.time_mgr_mut();
                        time_mgr.update();
                    }

                    {
                        let mut input_mgr = self.ctx.input_mgr_mut();
                        input_mgr.poll();
                    }

                    self.ctx.event_mgr().dispatch(&event_types::Update);

                    make_ui_scaler_dirty.run_now(&self.ctx.world());
                    update_ui_scaler.run_now(&self.ctx.world());
                    update_ui_element.run_now(&self.ctx.world());
                    update_ui_raycast_grid.run_now(&self.ctx.world());

                    self.ctx.ui_event_mgr_mut().handle_mouse_move();

                    {
                        let world = self.ctx.world();
                        let mut object_mgr = self.ctx.object_mgr_mut();
                        let object_hierarchy = object_mgr.object_hierarchy_mut();

                        object_hierarchy.copy_dirty_to_current_frame();

                        let transforms = world.read_component::<Transform>();
                        object_hierarchy.update_object_matrices(|entity| transforms.get(entity));
                    }

                    self.ctx.event_mgr().dispatch(&event_types::LateUpdate);

                    if window_occluded {
                        return;
                    }

                    update_camera_transform_buffer_system.run_now(&self.ctx.world());
                    render_system.run_now(&self.ctx.world());

                    return;
                }
                Event::RedrawRequested(id) if id == window_id => {
                    if loop_mode == EngineLoopMode::Poll {
                        return;
                    }

                    {
                        let mut time_mgr = self.ctx.time_mgr_mut();
                        time_mgr.update();
                    }

                    {
                        let mut input_mgr = self.ctx.input_mgr_mut();
                        input_mgr.poll();
                    }

                    self.ctx.event_mgr().dispatch(&event_types::Update);

                    make_ui_scaler_dirty.run_now(&self.ctx.world());
                    update_ui_scaler.run_now(&self.ctx.world());
                    update_ui_element.run_now(&self.ctx.world());
                    update_ui_raycast_grid.run_now(&self.ctx.world());

                    self.ctx.ui_event_mgr_mut().handle_mouse_move();

                    {
                        let world = self.ctx.world();
                        let mut object_mgr = self.ctx.object_mgr_mut();
                        let object_hierarchy = object_mgr.object_hierarchy_mut();

                        object_hierarchy.copy_dirty_to_current_frame();

                        let transforms = world.read_component::<Transform>();
                        object_hierarchy.update_object_matrices(|entity| transforms.get(entity));
                    }

                    self.ctx.event_mgr().dispatch(&event_types::LateUpdate);

                    update_camera_transform_buffer_system.run_now(&self.ctx.world());
                    render_system.run_now(&self.ctx.world());

                    return;
                }
                Event::WindowEvent {
                    event: WindowEvent::Occluded(occluded),
                    window_id: id,
                } if id == window_id => {
                    window_occluded = occluded;

                    return;
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    window_id: id,
                } if id == window_id => {
                    self.ctx
                        .input_mgr_mut()
                        .keyboard_mut()
                        .handle_window_event(input);

                    return;
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorEntered { .. },
                    window_id: id,
                } if id == window_id => {
                    // TODO: Handle cursor entered event here.

                    return;
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorLeft { .. },
                    window_id: id,
                } if id == window_id => {
                    self.ctx.ui_event_mgr_mut().handle_mouse_leave();

                    return;
                }
                Event::WindowEvent {
                    event: event @ WindowEvent::CursorMoved { .. },
                    window_id: id,
                } if id == window_id => {
                    self.ctx
                        .input_mgr_mut()
                        .mouse_mut()
                        .handle_window_event(&event);

                    if let WindowEvent::CursorMoved { position, .. } = &event {
                        let position =
                            position.to_logical::<f32>(self.ctx.screen_mgr().scale_factor());
                        self.ctx
                            .ui_event_mgr_mut()
                            .update_mouse_position(Vec2::new(position.x, position.y));
                    }

                    return;
                }
                Event::WindowEvent {
                    event: event @ WindowEvent::MouseInput { .. },
                    window_id: id,
                } if id == window_id => {
                    self.ctx
                        .input_mgr_mut()
                        .mouse_mut()
                        .handle_window_event(&event);

                    return;
                }
                Event::WindowEvent {
                    event: event @ WindowEvent::MouseWheel { .. },
                    window_id: id,
                } if id == window_id => {
                    self.ctx
                        .input_mgr_mut()
                        .mouse_mut()
                        .handle_window_event(&event);

                    return;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(inner_size),
                    window_id: id,
                } if id == window_id => {
                    self.ctx.screen_mgr_mut().update_size(inner_size);

                    if inner_size.width == 0 || inner_size.height == 0 {
                        window_occluded = true;
                        return;
                    } else {
                        window_occluded = false;
                    }

                    self.ctx.gfx_ctx().device.poll(MaintainBase::Wait);
                    self.ctx.gfx_ctx().resize(inner_size);
                    self.ctx.render_mgr_mut().resize(inner_size);

                    return;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            new_inner_size,
                        },
                    window_id: id,
                } if id == window_id => {
                    target_frame_interval.update_window(&self.ctx.window);
                    self.ctx
                        .screen_mgr_mut()
                        .update_scale_factor(scale_factor, *new_inner_size);

                    if new_inner_size.width == 0 || new_inner_size.height == 0 {
                        window_occluded = true;
                        return;
                    } else {
                        window_occluded = false;
                    }

                    self.ctx.gfx_ctx().resize(*new_inner_size);
                    self.ctx.render_mgr_mut().resize(*new_inner_size);

                    return;
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id: id,
                } if id == window_id => {
                    *control_flow = ControlFlow::Exit;

                    return;
                }
                _ => return,
            }
        })
    }
}

pub struct EngineConfig {
    pub title: String,
    pub resizable: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(Error, Debug)]
pub enum EngineInitError {
    #[error("winit os error: {0}")]
    WinitOsError(#[from] winit::error::OsError),
    #[error("winit external error: {0}")]
    WinitExternalError(#[from] winit::error::ExternalError),
    #[error("winit not supported error: {0}")]
    WinitNotSupportedError(#[from] winit::error::NotSupportedError),
    #[error("gfx context creation error: {0}")]
    GfxContextCreationError(#[from] GfxContextCreationError),
}

#[derive(Error, Debug)]
pub enum EngineExecError {
    #[error("gfx surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineLoopMode {
    Poll,
    Wait,
}

impl Default for EngineLoopMode {
    fn default() -> Self {
        Self::Poll
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineTargetFps {
    VSync,
    MilliHertz(NonZeroU32),
    Unlimited,
}

impl Default for EngineTargetFps {
    fn default() -> Self {
        Self::VSync
    }
}
