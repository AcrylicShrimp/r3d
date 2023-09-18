use super::ObjectId;
use crate::{math::Mat4, transform::Transform};
use bitvec::prelude::*;
use specs::prelude::*;
use std::{cmp::Ordering, ops::Range};

#[derive(Debug, Clone, Copy, Eq, Ord, Hash)]
pub struct ObjectSpan {
    pub index: u32,
    pub count: u32,
}

impl ObjectSpan {
    pub fn to_range(self) -> Range<usize> {
        self.index as usize..(self.index + self.count) as usize
    }
}

impl PartialEq for ObjectSpan {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for ObjectSpan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl From<ObjectSpan> for Range<u32> {
    fn from(span: ObjectSpan) -> Self {
        span.index..span.index + span.count
    }
}

impl From<ObjectSpan> for Range<usize> {
    fn from(span: ObjectSpan) -> Self {
        span.index as usize..(span.index + span.count) as usize
    }
}

#[derive(Debug, Clone)]
pub struct ObjectSiblingIter<'a> {
    parent: ObjectId,
    object: ObjectId,
    sibling_index: usize,
    object_spans: &'a [ObjectSpan],
    objects: &'a [ObjectId],
}

impl<'a> ObjectSiblingIter<'a> {
    pub fn new(
        parent: Option<ObjectId>,
        object: ObjectId,
        object_spans: &'a [ObjectSpan],
        objects: &'a [ObjectId],
    ) -> Self {
        Self {
            parent: parent.unwrap_or(object),
            object,
            sibling_index: 0,
            object_spans,
            objects,
        }
    }
}

impl<'a> Iterator for ObjectSiblingIter<'a> {
    type Item = ObjectId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.parent == self.object {
            return if self.sibling_index == 0 {
                self.sibling_index += 1;
                Some(self.object)
            } else {
                None
            };
        }

        let parent_span = self.object_spans[self.parent.get() as usize];
        let parent_span_index = parent_span.index as usize;
        let parent_span_count = parent_span.count as usize;
        let index = parent_span_index + 1 + self.sibling_index;

        if index < parent_span_index + parent_span_count {
            let object = self.objects[index];
            self.sibling_index += self.object_spans[object.get() as usize].count as usize;
            Some(object)
        } else {
            None
        }
    }
}

/// This represents a hierarchy of objects. It is used to store the parent-child relationships and keep track of the object order.
#[derive(Debug)]
pub struct ObjectHierarchy {
    // ordered
    objects: Vec<ObjectId>,
    object_entities: Vec<Entity>,
    object_dirties: BitVec,
    object_current_frame_dirties: BitVec,
    object_actives: BitVec,
    object_active_selfs: BitVec,
    // unordered
    object_spans: Vec<ObjectSpan>,
    object_parents: Vec<Vec<ObjectId>>,
    object_matrices: Vec<Mat4>,
}

impl ObjectHierarchy {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn objects(&self) -> &[ObjectId] {
        &self.objects
    }

    pub fn entities(&self) -> &[Entity] {
        &self.object_entities
    }

    pub fn index(&self, object: ObjectId) -> u32 {
        self.object_spans[object.get() as usize].index
    }

    pub fn entity(&self, object: ObjectId) -> Entity {
        self.object_entities[self.object_spans[object.get() as usize].index as usize]
    }

    pub fn is_dirty(&self, object: ObjectId) -> bool {
        self.object_dirties[self.object_spans[object.get() as usize].index as usize]
    }

    pub fn is_current_frame_dirty(&self, object: ObjectId) -> bool {
        self.object_current_frame_dirties[self.object_spans[object.get() as usize].index as usize]
    }

    pub fn is_active(&self, object: ObjectId) -> bool {
        self.object_actives[self.object_spans[object.get() as usize].index as usize]
    }

    pub fn is_active_self(&self, object: ObjectId) -> bool {
        self.object_active_selfs[self.object_spans[object.get() as usize].index as usize]
    }

    pub fn parent(&self, object: ObjectId) -> Option<ObjectId> {
        self.object_parents[object.get() as usize].first().copied()
    }

    pub fn parents(&self, object: ObjectId) -> &[ObjectId] {
        &self.object_parents[object.get() as usize]
    }

    pub fn children(&self, object: ObjectId) -> &[ObjectId] {
        let span = self.object_spans[object.get() as usize];
        &self.objects[(span.index + 1) as usize..(span.index + span.count) as usize]
    }

    pub fn matrix(&self, object: ObjectId) -> &Mat4 {
        &self.object_matrices[object.get() as usize]
    }

    pub fn matrix_mut(&mut self, object: ObjectId) -> &mut Mat4 {
        &mut self.object_matrices[object.get() as usize]
    }

    pub fn object_and_children(&self, object: ObjectId) -> &[ObjectId] {
        let span = self.object_spans[object.get() as usize];
        &self.objects[span.index as usize..(span.index + span.count) as usize]
    }

    pub fn sibling_iter(&self, object: ObjectId) -> ObjectSiblingIter {
        ObjectSiblingIter::new(
            self.parent(object),
            object,
            &self.object_spans,
            &self.objects,
        )
    }

    pub fn direct_children_iter(&self, object: ObjectId) -> Option<ObjectSiblingIter> {
        let span = self.object_spans[object.get() as usize];
        if span.count < 2 {
            None
        } else {
            Some(ObjectSiblingIter::new(
                Some(object),
                self.objects[span.index as usize + 1],
                &self.object_spans,
                &self.objects,
            ))
        }
    }

    pub fn set_dirty(&mut self, object: ObjectId) {
        self.object_dirties.as_mut_bitslice()[self.object_spans[object.get() as usize].to_range()]
            .fill(true);
    }

    pub fn copy_dirty_to_current_frame(&mut self) {
        self.object_current_frame_dirties
            .copy_from_bitslice(&self.object_dirties);
    }

    pub fn set_active(&mut self, object: ObjectId, is_active: bool) {
        self.object_active_selfs
            .set(object.get() as usize, is_active);

        let is_parent_active = match self.parent(object) {
            Some(parent) => self.is_active(parent),
            _ => true,
        };

        if is_active && is_parent_active {
            let children = self.children(object);
            let mut flags: BitVec = BitVec::with_capacity(children.len() + 1);

            flags.push(true);

            let base_index = self.object_spans[object.get() as usize].index;

            for &child in children {
                let is_parent_active = match self.parent(child) {
                    Some(parent) => {
                        let parent_index = self.object_spans[parent.get() as usize].index;
                        let index = parent_index - base_index;
                        flags[index as usize]
                    }
                    None => true,
                };
                flags.push(is_parent_active && self.is_active_self(child));
            }

            self.object_actives.as_mut_bitslice()
                [self.object_spans[object.get() as usize].to_range()]
            .copy_from_bitslice(&flags);
        } else {
            self.object_actives.as_mut_bitslice()
                [self.object_spans[object.get() as usize].to_range()]
            .fill(false);
        }
    }

    pub fn reset_dirties(&mut self) {
        self.object_dirties.fill(false);
    }

    /// Adds the given object to the hierarchy.
    pub fn add(&mut self, object: ObjectId, entity: Entity) {
        let object_usize = object.get() as usize;

        if object_usize < self.object_spans.len() {
            self.object_spans[object_usize] = ObjectSpan {
                index: self.objects.len() as u32,
                count: 1,
            };
            self.object_parents[object_usize].clear();
        } else {
            debug_assert!(object_usize == self.object_spans.len());
            self.object_spans.push(ObjectSpan {
                index: self.objects.len() as u32,
                count: 1,
            });
            self.object_parents.push(Vec::with_capacity(4));
            self.object_matrices.push(Mat4::identity());
        }

        self.objects.push(object);
        self.object_entities.push(entity);
        self.object_dirties.push(true);
        self.object_current_frame_dirties.push(true);
        self.object_actives.push(true);
        self.object_active_selfs.push(true);
    }

    /// Removes the given object and its children. Returns the removed objects in the order of hierarchy.
    pub fn remove(&mut self, object: ObjectId) -> Vec<Entity> {
        let object_usize = object.get() as usize;
        let span = self.object_spans[object_usize];
        let to_be_removed = self.object_entities[span.to_range()].to_vec();

        // Remove the object and its children from its parents.
        for &parent in &self.object_parents[object_usize] {
            let parent_usize = parent.get() as usize;
            self.object_spans[parent_usize].count -= span.count;
        }

        let span_index = span.index as usize;
        let span_count = span.count as usize;

        // Remove the object and its children from the ordered objects.
        for &object in &self.objects[span_index + span_count..] {
            self.object_spans[object.get() as usize].index -= span.count;
        }

        if span_index + span_count < self.objects.len() {
            self.objects
                .copy_within(span_index + span_count.., span_index);
        }

        self.objects.truncate(self.objects.len() - span_count);

        if span_index + span_count < self.object_entities.len() {
            self.object_entities
                .copy_within(span_index + span_count.., span_index);
        }

        self.object_entities
            .truncate(self.object_entities.len() - span_count);

        if span_index + span_count < self.object_dirties.len() {
            self.object_dirties
                .copy_within(span_index + span_count.., span_index);
        }

        self.object_dirties
            .truncate(self.object_dirties.len() - span_count);

        if span_index + span_count < self.object_current_frame_dirties.len() {
            self.object_current_frame_dirties
                .copy_within(span_index + span_count.., span_index);
        }

        self.object_current_frame_dirties
            .truncate(self.object_current_frame_dirties.len() - span_count);

        if span_index + span_count < self.object_actives.len() {
            self.object_actives
                .copy_within(span_index + span_count.., span_index);
        }

        self.object_actives
            .truncate(self.object_actives.len() - span_count);

        if span_index + span_count < self.object_active_selfs.len() {
            self.object_active_selfs
                .copy_within(span_index + span_count.., span_index);
        }

        self.object_active_selfs
            .truncate(self.object_active_selfs.len() - span_count);

        to_be_removed
    }

    /// Sets the parent of the given object and re-order all objects.
    pub fn set_parent(&mut self, object: ObjectId, parent: Option<ObjectId>) {
        self.set_dirty(object);

        let object_usize = object.get() as usize;
        let span = self.object_spans[object_usize];

        // Remove the object and its children from its parents.
        for &parent in &self.object_parents[object_usize] {
            let parent_usize = parent.get() as usize;
            self.object_spans[parent_usize].count -= span.count;
        }

        let parent_count = self.object_parents[object_usize].len();

        // Remove the parents of the object and its children.
        for &object in &self.objects[span.to_range()] {
            let parents = &mut self.object_parents[object.get() as usize];
            parents.truncate(parents.len() - parent_count);
        }

        let destination_index = if let Some(parent) = parent {
            let parent_usize = parent.get() as usize;
            let (left, right) = self.object_parents.split_at_mut(parent_usize);
            let (high_parents, right) = right.split_first_mut().unwrap();

            // Assign a new parent and its parents.
            for &object in &self.objects[span.to_range()] {
                let parents = if object < parent {
                    &mut left[object.get() as usize]
                } else {
                    &mut right[object.get() as usize - parent_usize - 1]
                };
                parents.reserve(high_parents.len() + 1);
                parents.push(parent);
                parents.extend_from_slice(high_parents);
            }

            let prev_parent_span = self.object_spans[parent_usize];

            // Add the object and its children to its new parent.
            self.object_spans[parent_usize].count += span.count;

            for &high_parent in high_parents.iter() {
                let high_parent_usize = high_parent.get() as usize;
                self.object_spans[high_parent_usize].count += span.count;
            }

            (prev_parent_span.index + prev_parent_span.count) as usize
        } else {
            self.objects.len()
        };

        // Move the object and its children to the new destination.
        self.move_objects(object, destination_index);

        // Set dirties.
        self.set_dirty(object);

        // Update active flags.
        self.set_active(object, self.is_active_self(object));
    }

    /// Updates the object matrices.
    pub fn update_object_matrices<'a>(
        &mut self,
        transforms: impl Fn(Entity) -> Option<&'a Transform>,
    ) {
        for (&object, &entity) in self.objects.iter().zip(self.object_entities.iter()) {
            if !self.is_dirty(object) {
                continue;
            }

            let mut matrix = if let Some(transform) = transforms(entity) {
                transform.matrix()
            } else {
                Mat4::identity()
            };

            if let Some(parent) = self.parent(object) {
                matrix *= self.matrix(parent);
            }

            self.object_matrices[object.get() as usize] = matrix;
        }

        self.reset_dirties();
    }

    /// Moves the given object and its children to the destination index.
    fn move_objects(&mut self, object: ObjectId, destination_index: usize) {
        let object = object.get() as usize;
        let span = self.object_spans[object];
        let span_index = span.index as usize;
        let span_count = span.count as usize;
        let span_index_end = span_index + span_count;

        if destination_index == span_index {
            return;
        }

        if destination_index < span_index {
            let offset = (span_index - destination_index) as u32;

            for &object in &self.objects[span_index..span_index_end] {
                self.object_spans[object.get() as usize].index -= offset;
            }

            for &object in &self.objects[destination_index..span_index] {
                self.object_spans[object.get() as usize].index += span.count;
            }

            self.swap_range(destination_index, span_index, span_index_end);
        } else {
            let offset = (destination_index - span_index - span_count) as u32;

            for &object in &self.objects[span_index..span_index_end] {
                self.object_spans[object.get() as usize].index += offset;
            }

            for &object in &self.objects[span_index_end..destination_index] {
                self.object_spans[object.get() as usize].index -= span.count;
            }

            self.swap_range(span_index, span_index_end, destination_index);
        }
    }

    /// Swaps the given two range index_left..index_mid and index_mid..index_right.
    fn swap_range(&mut self, index_left: usize, index_mid: usize, index_right: usize) {
        debug_assert!(index_left <= index_mid);
        debug_assert!(index_mid <= index_right);

        let (temp, temp_dest, src, dest) = if index_mid - index_left < index_right - index_mid {
            (
                index_left..index_mid,
                index_right - (index_mid - index_left),
                index_mid..index_right,
                index_left,
            )
        } else {
            (
                index_mid..index_right,
                index_left,
                index_left..index_mid,
                index_right - (index_mid - index_left),
            )
        };

        let temp_objects = self.objects[temp.clone()].to_vec();
        self.objects.copy_within(src.clone(), dest);
        self.objects[temp_dest..temp_dest + temp.len()].copy_from_slice(&temp_objects);

        let temp_object_entities = self.object_entities[temp.clone()].to_vec();
        self.object_entities.copy_within(src.clone(), dest);
        self.object_entities[temp_dest..temp_dest + temp.len()]
            .copy_from_slice(&temp_object_entities);

        let temp_object_dirties = self.object_dirties[temp.clone()].to_bitvec();
        self.object_dirties.copy_within(src.clone(), dest);
        self.object_dirties[temp_dest..temp_dest + temp.len()]
            .copy_from_bitslice(&temp_object_dirties);

        let temp_object_current_frame_dirties =
            self.object_current_frame_dirties[temp.clone()].to_bitvec();
        self.object_current_frame_dirties
            .copy_within(src.clone(), dest);
        self.object_current_frame_dirties[temp_dest..temp_dest + temp.len()]
            .copy_from_bitslice(&temp_object_current_frame_dirties);

        let temp_object_actives = self.object_actives[temp.clone()].to_bitvec();
        self.object_actives.copy_within(src.clone(), dest);
        self.object_actives[temp_dest..temp_dest + temp.len()]
            .copy_from_bitslice(&temp_object_actives);

        let temp_object_active_selfs = self.object_active_selfs[temp.clone()].to_bitvec();
        self.object_active_selfs.copy_within(src.clone(), dest);
        self.object_active_selfs[temp_dest..temp_dest + temp.len()]
            .copy_from_bitslice(&temp_object_active_selfs);
    }
}

impl Default for ObjectHierarchy {
    fn default() -> Self {
        Self {
            objects: Vec::with_capacity(1024),
            object_entities: Vec::with_capacity(1024),
            object_dirties: BitVec::with_capacity(1024),
            object_current_frame_dirties: BitVec::with_capacity(1024),
            object_actives: BitVec::with_capacity(1024),
            object_active_selfs: BitVec::with_capacity(1024),

            object_spans: Vec::with_capacity(1024),
            object_parents: Vec::with_capacity(1024),
            object_matrices: Vec::with_capacity(1024),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::math::Vec3;
    use std::collections::HashMap;

    fn equals_float(a: f32, b: f32) -> bool {
        (a - b).abs() <= f32::EPSILON
    }

    fn equals_mat4(a: &Mat4, b: &Mat4) -> bool {
        for i in 0..16 {
            if !equals_float(a.elements[i], b.elements[i]) {
                return false;
            }
        }

        true
    }

    fn create_hierarchy(object_count: u32) -> ObjectHierarchy {
        let mut hierarchy = ObjectHierarchy::new();
        let mut world = World::new();

        for id in 0..object_count {
            hierarchy.add(ObjectId::from_u32(id), world.create_entity().build());
        }

        hierarchy
    }

    #[test]
    fn check_hierarchy_object_order() {
        let mut hierarchy = create_hierarchy(4);

        assert_eq!(
            hierarchy.objects(),
            &[
                ObjectId::from_u32(0),
                ObjectId::from_u32(1),
                ObjectId::from_u32(2),
                ObjectId::from_u32(3),
            ]
        );

        hierarchy.set_parent(ObjectId::from_u32(2), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(3), Some(ObjectId::from_u32(0)));

        assert_eq!(
            hierarchy.objects(),
            &[
                ObjectId::from_u32(0),
                ObjectId::from_u32(2),
                ObjectId::from_u32(3),
                ObjectId::from_u32(1),
            ]
        );

        hierarchy.set_parent(ObjectId::from_u32(2), Some(ObjectId::from_u32(1)));
        hierarchy.set_parent(ObjectId::from_u32(3), Some(ObjectId::from_u32(1)));

        assert_eq!(
            hierarchy.objects(),
            &[
                ObjectId::from_u32(0),
                ObjectId::from_u32(1),
                ObjectId::from_u32(2),
                ObjectId::from_u32(3),
            ]
        );

        hierarchy.set_parent(ObjectId::from_u32(0), Some(ObjectId::from_u32(1)));

        assert_eq!(
            hierarchy.objects(),
            &[
                ObjectId::from_u32(1),
                ObjectId::from_u32(2),
                ObjectId::from_u32(3),
                ObjectId::from_u32(0),
            ]
        );
    }

    #[test]
    fn check_hierarchy_object_matrix() {
        let mut hierarchy = create_hierarchy(4);

        hierarchy.matrix_mut(ObjectId::from_u32(0)).elements[0] = 100.0;
        hierarchy.matrix_mut(ObjectId::from_u32(1)).elements[0] = 200.0;
        hierarchy.matrix_mut(ObjectId::from_u32(2)).elements[0] = 300.0;
        hierarchy.matrix_mut(ObjectId::from_u32(3)).elements[0] = 400.0;

        hierarchy.set_parent(ObjectId::from_u32(0), Some(ObjectId::from_u32(3)));
        hierarchy.set_parent(ObjectId::from_u32(1), Some(ObjectId::from_u32(2)));

        assert_eq!(
            hierarchy.objects(),
            &[
                ObjectId::from_u32(2),
                ObjectId::from_u32(1),
                ObjectId::from_u32(3),
                ObjectId::from_u32(0),
            ]
        );

        assert!(equals_float(
            hierarchy.matrix(ObjectId::from_u32(0)).elements[0],
            100.0
        ));
        assert!(equals_float(
            hierarchy.matrix(ObjectId::from_u32(1)).elements[0],
            200.0
        ));
        assert!(equals_float(
            hierarchy.matrix(ObjectId::from_u32(2)).elements[0],
            300.0
        ));
        assert!(equals_float(
            hierarchy.matrix(ObjectId::from_u32(3)).elements[0],
            400.0
        ));
    }

    #[test]
    fn check_hierarchy_object_dirty() {
        let mut hierarchy = create_hierarchy(1);

        assert_eq!(hierarchy.is_dirty(ObjectId::from_u32(0)), true);

        hierarchy.reset_dirties();

        assert_eq!(hierarchy.is_dirty(ObjectId::from_u32(0)), false);
    }

    #[test]
    fn check_hierarchy_object_removal() {
        let mut hierarchy = create_hierarchy(6);

        hierarchy.remove(ObjectId::from_u32(1));
        hierarchy.remove(ObjectId::from_u32(4));

        assert_eq!(
            hierarchy.objects(),
            &[
                ObjectId::from_u32(0),
                ObjectId::from_u32(2),
                ObjectId::from_u32(3),
                ObjectId::from_u32(5),
            ]
        );

        hierarchy.set_parent(ObjectId::from_u32(0), Some(ObjectId::from_u32(5)));
        let to_be_removed = vec![
            hierarchy.entity(ObjectId::from_u32(5)),
            hierarchy.entity(ObjectId::from_u32(0)),
        ];
        let removed = hierarchy.remove(ObjectId::from_u32(5));

        assert_eq!(removed, to_be_removed);
        assert_eq!(
            hierarchy.objects(),
            &[ObjectId::from_u32(2), ObjectId::from_u32(3),]
        );
    }

    #[test]
    fn check_hierarchy_object_active_flag() {
        let mut hierarchy = create_hierarchy(10);

        hierarchy.set_parent(ObjectId::from_u32(1), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(2), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(3), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(4), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(5), Some(ObjectId::from_u32(1)));
        hierarchy.set_parent(ObjectId::from_u32(6), Some(ObjectId::from_u32(2)));
        hierarchy.set_parent(ObjectId::from_u32(7), Some(ObjectId::from_u32(3)));
        hierarchy.set_parent(ObjectId::from_u32(8), Some(ObjectId::from_u32(4)));

        hierarchy.set_active(ObjectId::from_u32(1), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(4)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(5)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(6)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(7)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(8)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(9)), true);

        hierarchy.set_active(ObjectId::from_u32(0), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(4)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(5)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(6)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(7)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(8)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(9)), true);

        hierarchy.set_active(ObjectId::from_u32(1), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(4)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(5)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(6)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(7)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(8)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(9)), true);

        hierarchy.set_active(ObjectId::from_u32(0), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(4)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(5)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(6)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(7)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(8)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(9)), true);
    }

    #[test]
    fn check_hierarchy_object_active_flag_change_parent() {
        let mut hierarchy = create_hierarchy(4);

        hierarchy.set_active(ObjectId::from_u32(0), true);
        hierarchy.set_active(ObjectId::from_u32(1), false);

        hierarchy.set_parent(ObjectId::from_u32(2), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(3), Some(ObjectId::from_u32(2)));

        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), true);

        hierarchy.set_parent(ObjectId::from_u32(2), Some(ObjectId::from_u32(1)));

        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), false);

        hierarchy.set_parent(ObjectId::from_u32(2), None);

        assert_eq!(hierarchy.is_active(ObjectId::from_u32(0)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(1)), false);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(2)), true);
        assert_eq!(hierarchy.is_active(ObjectId::from_u32(3)), true);
    }

    #[test]
    fn check_hierarchy_object_matrix_update_uniform_scales() {
        let mut hierarchy = create_hierarchy(4);

        hierarchy.set_parent(ObjectId::from_u32(1), Some(ObjectId::from_u32(0)));
        hierarchy.set_parent(ObjectId::from_u32(2), Some(ObjectId::from_u32(1)));
        hierarchy.set_parent(ObjectId::from_u32(3), Some(ObjectId::from_u32(2)));

        let mut transforms = HashMap::new();
        transforms.insert(hierarchy.entity(ObjectId::from_u32(0)), {
            let mut transform = Transform::new();
            transform.scale = Vec3::new(0.5, 0.5, 0.5);
            transform
        });
        transforms.insert(hierarchy.entity(ObjectId::from_u32(1)), {
            let mut transform = Transform::new();
            transform.scale = Vec3::new(0.5, 0.5, 0.5);
            transform
        });
        transforms.insert(hierarchy.entity(ObjectId::from_u32(2)), {
            let mut transform = Transform::new();
            transform.scale = Vec3::new(0.5, 0.5, 0.5);
            transform
        });
        transforms.insert(hierarchy.entity(ObjectId::from_u32(3)), {
            let mut transform = Transform::new();
            transform.scale = Vec3::new(0.5, 0.5, 0.5);
            transform
        });

        hierarchy.update_object_matrices(|entity| transforms.get(&entity));

        assert_eq!(
            equals_mat4(
                hierarchy.matrix(ObjectId::from_u32(0)),
                &(Mat4::scale(Vec3::ONE * 0.5))
            ),
            true
        );
        assert_eq!(
            equals_mat4(
                hierarchy.matrix(ObjectId::from_u32(1)),
                &(Mat4::scale(Vec3::ONE * 0.5 * 0.5))
            ),
            true
        );
        assert_eq!(
            equals_mat4(
                hierarchy.matrix(ObjectId::from_u32(2)),
                &(Mat4::scale(Vec3::ONE * 0.5 * 0.5 * 0.5))
            ),
            true
        );
        assert_eq!(
            equals_mat4(
                hierarchy.matrix(ObjectId::from_u32(3)),
                &(Mat4::scale(Vec3::ONE * 0.5 * 0.5 * 0.5 * 0.5))
            ),
            true
        );
    }
}
