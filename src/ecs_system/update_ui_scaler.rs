use crate::{
    gfx::ScreenManager,
    math::Vec3,
    object::Object,
    transform::Transform,
    ui::{UIScaleMode, UIScaler, UISize},
    ContextHandle,
};
use specs::prelude::*;
use std::cmp::Ordering;

pub struct UpdateUIScaler {
    ctx: ContextHandle,
}

impl UpdateUIScaler {
    pub fn new(ctx: ContextHandle) -> Self {
        Self { ctx }
    }
}

impl<'a> System<'a> for UpdateUIScaler {
    type SystemData = (
        ReadStorage<'a, Object>,
        ReadStorage<'a, UIScaler>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, UISize>,
    );

    fn run(&mut self, (objects, scalers, mut transforms, mut sizes): Self::SystemData) {
        let object_mgr = self.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();

        let mut pairs = Vec::from_iter((&objects, &scalers).join().filter_map(|(object, _)| {
            if !hierarchy.is_dirty(object.object_id()) {
                return None;
            }

            Some(Pair {
                index: hierarchy.index(object.object_id()),
                parent: hierarchy
                    .parent(object.object_id())
                    .map(|object_id| hierarchy.entity(object_id)),
                child: hierarchy.entity(object.object_id()),
            })
        }));
        pairs.sort_unstable();

        let screen_mgr = self.ctx.screen_mgr();

        for pair in pairs {
            compute_pair(pair, &screen_mgr, &scalers, &mut transforms, &mut sizes);
        }
    }
}

struct Pair {
    pub index: u32,
    pub parent: Option<Entity>,
    pub child: Entity,
}

impl PartialEq for Pair {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for Pair {}

impl PartialOrd for Pair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl Ord for Pair {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

fn compute_pair(
    pair: Pair,
    screen_mgr: &ScreenManager,
    scalers: &ReadStorage<UIScaler>,
    transforms: &mut WriteStorage<Transform>,
    sizes: &mut WriteStorage<UISize>,
) {
    let (target_width, target_height) =
        match pair.parent.and_then(|parent| sizes.get(parent).cloned()) {
            Some(size) => (size.width, size.height),
            None => (screen_mgr.width() as f32, screen_mgr.height() as f32),
        };

    let scaler = scalers.get(pair.child).cloned().unwrap();
    let (width, height) = match scaler.mode {
        UIScaleMode::Constant => (scaler.reference_size.x, scaler.reference_size.y),
        UIScaleMode::Stretch => (target_width, target_height),
        UIScaleMode::Fit => {
            let scale_x = target_width / scaler.reference_size.x;
            let scale_y = target_height / scaler.reference_size.y;
            let scale = f32::min(scale_x, scale_y);
            (
                scale * scaler.reference_size.x,
                scale * scaler.reference_size.y,
            )
        }
        UIScaleMode::Fill => {
            let scale_x = target_width / scaler.reference_size.x;
            let scale_y = target_height / scaler.reference_size.y;
            let scale = f32::max(scale_x, scale_y);
            (
                scale * scaler.reference_size.x,
                scale * scaler.reference_size.y,
            )
        }
        UIScaleMode::MatchWidth => {
            let scale = target_width / scaler.reference_size.x;
            (
                scale * scaler.reference_size.x,
                scale * scaler.reference_size.y,
            )
        }
        UIScaleMode::MatchHeight => {
            let scale = target_height / scaler.reference_size.y;
            (
                scale * scaler.reference_size.x,
                scale * scaler.reference_size.y,
            )
        }
    };

    let transform = transforms.get_mut(pair.child).unwrap();
    transform.position = Vec3::new(width * -0.5f32, height * -0.5f32, 0.0f32);

    let size = sizes.get_mut(pair.child).unwrap();
    size.width = width;
    size.height = height;
}
