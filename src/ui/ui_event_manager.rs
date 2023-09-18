use crate::{
    math::Vec2,
    object::ObjectHandle,
    object_event::object_event_types::{MouseEnterEvent, MouseLeaveEvent, MouseMoveEvent},
    use_context,
};

pub struct UIEventManager {
    prev_object: Option<ObjectHandle>,
    mouse_position: Option<Vec2>,
    is_dirty: bool,
}

impl UIEventManager {
    pub fn new() -> Self {
        Self {
            prev_object: None,
            mouse_position: None,
            is_dirty: false,
        }
    }

    pub fn update_mouse_position(&mut self, point: Vec2) {
        let screen_mgr = use_context().screen_mgr();
        let screen_size = Vec2::new(screen_mgr.width() as f32, screen_mgr.height() as f32);
        let point = Vec2::new(
            point.x - screen_size.x * 0.5f32,
            screen_size.y * 0.5f32 - point.y,
        );
        self.mouse_position = Some(point);
        self.is_dirty = true;
    }

    pub fn remove_object(&mut self, object: &ObjectHandle) {
        if let Some(prev_object) = self.prev_object.as_ref() {
            if prev_object == object {
                self.prev_object = None;
                self.is_dirty = true;
            }
        }
    }

    pub fn handle_mouse_leave(&mut self) {
        if let Some(prev_object) = self.prev_object.as_ref() {
            use_context()
                .object_event_mgr()
                .dispatch(prev_object.object_id, &MouseLeaveEvent);
            self.prev_object = None;
            self.is_dirty = false;
        }
    }

    pub fn handle_mouse_move(&mut self) {
        if !self.is_dirty {
            return;
        }

        let point = if let Some(mouse_position) = self.mouse_position {
            mouse_position
        } else {
            return;
        };

        let current = use_context().ui_raycast_mgr_mut().raycast(point);
        let event_mgr = use_context().object_event_mgr();

        match (self.prev_object.as_ref(), current.as_ref()) {
            (Some(prev), Some(current)) if prev == current => {
                event_mgr.dispatch(current.object_id, &MouseMoveEvent);
            }
            (Some(prev), Some(current)) => {
                event_mgr.dispatch(prev.object_id, &MouseLeaveEvent);
                event_mgr.dispatch(current.object_id, &MouseEnterEvent);
            }
            (Some(prev), None) => {
                event_mgr.dispatch(prev.object_id, &MouseLeaveEvent);
            }
            (None, Some(current)) => {
                event_mgr.dispatch(current.object_id, &MouseEnterEvent);
            }
            _ => {}
        }

        self.prev_object = current;
        self.is_dirty = false;
    }
}
