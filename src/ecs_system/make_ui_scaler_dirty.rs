use crate::{object::Object, ui::UIScaler, ContextHandle};
use specs::prelude::*;

pub struct MakeUIScalerDirty {
    ctx: ContextHandle,
}

impl MakeUIScalerDirty {
    pub fn new(ctx: ContextHandle) -> Self {
        Self { ctx }
    }
}

impl<'a> System<'a> for MakeUIScalerDirty {
    type SystemData = (ReadStorage<'a, Object>, ReadStorage<'a, UIScaler>);

    fn run(&mut self, (objects, scalers): Self::SystemData) {
        let mut screen_mgr = self.ctx.screen_mgr_mut();

        if !screen_mgr.is_dirty() {
            return;
        }

        screen_mgr.reset_dirty();

        let mut object_mgr = self.ctx.object_mgr_mut();
        let hierarchy = object_mgr.object_hierarchy_mut();

        for (object, _) in (&objects, &scalers).join() {
            hierarchy.set_dirty(object.object_id());
        }
    }
}
