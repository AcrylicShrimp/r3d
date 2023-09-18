use super::UISizeComponent;
use crate::{
    math::{Vec2, Vec4},
    object::ObjectHandle,
    transform::TransformComponent,
};
use std::collections::{BTreeSet, HashMap};

/// Grid width in pixels.
pub const GRID_WIDTH: u64 = 128;
/// Grid height in pixels.
pub const GRID_HEIGHT: u64 = 128;

pub const MAX_SCREEN_WIDTH: u64 = GRID_WIDTH * i8::MAX as u64;
pub const MAX_SCREEN_HEIGHT: u64 = GRID_HEIGHT * i8::MAX as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellIndex {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellAddress {
    pub x: i8,
    pub y: i8,
    pub width: i8,
    pub height: i8,
}

impl CellAddress {
    pub fn to_indices_iter(self) -> impl Iterator<Item = CellIndex> {
        let CellAddress {
            x,
            y,
            width,
            height,
        } = self;
        (x..x + width).flat_map(move |x| (y..y + height).map(move |y| CellIndex { x, y }))
    }
}

#[derive(Clone)]
struct OrderedObject {
    pub index: u32,
    pub object: ObjectHandle,
}

impl PartialEq for OrderedObject {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for OrderedObject {}

impl PartialOrd for OrderedObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl Ord for OrderedObject {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

/// Maintains a grid of objects for fast raycasting. All objects added to the grid should be rebuilt when they have been changed.
/// Changes include:
/// - Dirty flag of transform component is set.
/// - Hierarchical order of object is changed e.g. parent is changed, other object is added to the parent, etc.
pub struct UIRaycastManager {
    objects: HashMap<ObjectHandle, CellAddress>,
    cells: HashMap<CellIndex, BTreeSet<OrderedObject>>,
}

impl UIRaycastManager {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            cells: HashMap::new(),
        }
    }

    /// Add an object to be raycasted.
    /// If the object is already added, it will be removed first.
    /// Order of objects is important, so the object with higher index (drawn later) will be raycasted first.
    pub fn add_object(&mut self, object: ObjectHandle) {
        self.remove_object(&object);

        let address = compute_aabb_cell_address(&object);
        self.objects.insert(object.clone(), address);

        let key = OrderedObject {
            index: object.index(),
            object: object.clone(),
        };

        for index in address.to_indices_iter() {
            self.cells.entry(index).or_default().insert(key.clone());
        }
    }

    /// Remove an object from grid.
    pub fn remove_object(&mut self, object: &ObjectHandle) {
        let address = if let Some(address) = self.objects.remove(&object) {
            address
        } else {
            return;
        };

        let key = OrderedObject {
            index: object.index(),
            object: object.clone(),
        };

        for index in address.to_indices_iter() {
            if let Some(cell) = self.cells.get_mut(&index) {
                cell.remove(&key);
            }
        }
    }

    /// Raycast a point.
    /// The point must in screen space, but origin is at center (x range `[-width/2, width/2]`, y range `[-height/2, height/2]`)
    pub fn raycast(&mut self, point: Vec2) -> Option<ObjectHandle> {
        let x = (point.x / GRID_WIDTH as f32).round() as i8;
        let y = (point.y / GRID_HEIGHT as f32).round() as i8;

        let cell = if let Some(cell) = self.cells.get(&CellIndex { x, y }) {
            cell
        } else {
            return None;
        };

        for object in cell {
            let inverse_matrix = object
                .component::<TransformComponent>()
                .world_inverse_matrix();
            let point: Vec2 = (Vec4::new(point.x, point.y, 0.0, 1.0) * &inverse_matrix).into();
            let size = object.component::<UISizeComponent>().size();

            if point.x >= -size.x && point.x <= size.x && point.y >= -size.y && point.y <= size.y {
                // TODO: Should we consider the alpha value of the object?
                return Some(object.clone());
            }
        }

        None
    }
}

fn compute_aabb_cell_address(object: &ObjectHandle) -> CellAddress {
    let aabb = compute_aabb(object);

    let x_min = (aabb.min.x / GRID_WIDTH as f32).round() as i8;
    let y_min = (aabb.min.y / GRID_HEIGHT as f32).round() as i8;
    let x_max = (aabb.max.x / GRID_WIDTH as f32).round() as i8;
    let y_max = (aabb.max.y / GRID_HEIGHT as f32).round() as i8;

    let width = x_max - x_min + 1;
    let height = y_max - y_min + 1;

    CellAddress {
        x: x_min,
        y: y_min,
        width,
        height,
    }
}

struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

fn compute_aabb(object: &ObjectHandle) -> AABB {
    let matrix = object.component::<TransformComponent>().world_matrix();
    let size = object.component::<UISizeComponent>().size();
    let points: [Vec2; 4] = [
        (Vec4::new(0.0, 0.0, 0.0, 1.0) * &matrix).into(),
        (Vec4::new(size.x, 0.0, 0.0, 1.0) * &matrix).into(),
        (Vec4::new(0.0, size.y, 0.0, 1.0) * &matrix).into(),
        (Vec4::new(size.x, size.y, 0.0, 1.0) * &matrix).into(),
    ];

    let min = points
        .iter()
        .fold(Vec2::new(f32::MAX, f32::MAX), |min, point| {
            Vec2::new(min.x.min(point.x), min.y.min(point.y))
        });
    let max = points
        .iter()
        .fold(Vec2::new(f32::MIN, f32::MIN), |max, point| {
            Vec2::new(max.x.max(point.x), max.y.max(point.y))
        });

    AABB { min, max }
}
