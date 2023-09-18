use crate::{
    math::Vec3,
    object::Object,
    transform::Transform,
    ui::{UIElement, UISize},
    ContextHandle,
};
use specs::prelude::*;
use std::cmp::Ordering;

pub struct UpdateUIElement {
    ctx: ContextHandle,
}

impl UpdateUIElement {
    pub fn new(ctx: ContextHandle) -> Self {
        Self { ctx }
    }
}

impl<'a> System<'a> for UpdateUIElement {
    type SystemData = (
        ReadStorage<'a, Object>,
        ReadStorage<'a, UIElement>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, UISize>,
    );

    fn run(&mut self, (objects, elements, mut transforms, mut sizes): Self::SystemData) {
        let object_mgr = self.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();

        let mut pairs = Vec::from_iter((&objects, &elements).join().filter_map(|(object, _)| {
            if !hierarchy.is_dirty(object.object_id()) {
                return None;
            }

            let parent = if let Some(parent) = hierarchy.parent(object.object_id()) {
                hierarchy.entity(parent)
            } else {
                return None;
            };

            Some(Pair {
                index: hierarchy.index(object.object_id()),
                parent,
                child: hierarchy.entity(object.object_id()),
            })
        }));
        pairs.sort_unstable();

        for pair in pairs {
            compute_pair(pair, &elements, &mut transforms, &mut sizes);
        }
    }
}

struct Pair {
    pub index: u32,
    pub parent: Entity,
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

/// Computes the position and size of the child element based on the parent element.
/// It is left-bottom based.
fn compute_pair(
    pair: Pair,
    elements: &ReadStorage<UIElement>,
    transforms: &mut WriteStorage<Transform>,
    sizes: &mut WriteStorage<UISize>,
) {
    let (parent_width, parent_height) = if let Some(parent) = sizes.get(pair.parent) {
        (parent.width, parent.height)
    } else {
        return;
    };
    let element = elements.get(pair.child).unwrap();

    let margin_left = parent_width * element.anchor.min.x;
    let margin_bottom = parent_height * element.anchor.min.y;
    let margin_right = parent_width * element.anchor.max.x;
    let margin_top = parent_height * element.anchor.max.y;

    let width = margin_right - margin_left - element.margin.left - element.margin.right;
    let height = margin_top - margin_bottom - element.margin.bottom - element.margin.top;

    let transform = transforms.get_mut(pair.child).unwrap();
    transform.position = Vec3::new(
        margin_left + element.margin.left,
        margin_bottom + element.margin.bottom,
        0.0,
    );

    let size = sizes.get_mut(pair.child).unwrap();
    size.width = width;
    size.height = height;
}
