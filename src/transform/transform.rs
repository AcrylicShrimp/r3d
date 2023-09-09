use crate::{
    math::{Mat4, Quat, Vec3, Vec4},
    object::{ObjectHierarchy, ObjectId},
};
use specs::{prelude::*, Component};

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_mat4(matrix: &Mat4) -> Self {
        let (position, rotation, scale) = matrix.split();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Returns the transform matrix that transforms from local space to world space.
    pub fn matrix(&self) -> Mat4 {
        Mat4::srt(self.position, self.rotation, self.scale)
    }

    /// Returns the inverse transform matrix that transforms from world space to local space.
    pub fn inverse_matrix(&self) -> Mat4 {
        Mat4::trs(-self.position, -self.rotation, Vec3::recip(self.scale))
    }

    /// Returns the world position of the given object.
    pub fn world_position(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let mut position = Vec4::from_vec3(self.position, 1.0);

        for &parent_id in hierarchy.parents(object_id) {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                position *= parent_transform.matrix();
            }
        }

        position.into()
    }

    /// Returns the world rotation of the given object.
    pub fn world_rotation(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Quat {
        let mut rotation = self.rotation;

        for &parent_id in hierarchy.parents(object_id) {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                rotation *= parent_transform.rotation;
            }
        }

        rotation
    }

    /// Returns the world scale of the given object.
    pub fn world_scale(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let mut scale = self.scale;

        for &parent_id in hierarchy.parents(object_id) {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                scale *= parent_transform.scale;
            }
        }

        scale
    }

    /// Sets the world position of the given object.
    pub fn set_world_position(
        &mut self,
        position: Vec3,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) {
        let mut position = Vec4::from_vec3(position, 1.0);

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                position *= parent_transform.inverse_matrix();
            }
        }

        self.position = position.into();
    }

    /// Sets the world rotation of the given object.
    pub fn set_world_rotation(
        &mut self,
        rotation: Quat,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) {
        let mut rotation = rotation;

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                rotation *= parent_transform.rotation.conjugated();
            }
        }

        self.rotation = rotation;
    }

    /// Sets the world scale of the given object.
    pub fn set_world_scale(
        &mut self,
        scale: Vec3,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) {
        let mut scale = scale;

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                scale /= parent_transform.scale;
            }
        }

        self.scale = scale;
    }

    /// Returns the world forward vector of the given object.
    pub fn forward(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let rotation = self.world_rotation(object_id, hierarchy, transforms);
        rotation * Vec3::FORWARD
    }

    /// Returns the world right vector of the given object.
    pub fn backward(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let rotation = self.world_rotation(object_id, hierarchy, transforms);
        rotation * Vec3::BACKWARD
    }

    /// Returns the world right vector of the given object.
    pub fn right(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let rotation = self.world_rotation(object_id, hierarchy, transforms);
        rotation * Vec3::RIGHT
    }

    /// Returns the world left vector of the given object.
    pub fn left(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let rotation = self.world_rotation(object_id, hierarchy, transforms);
        rotation * Vec3::LEFT
    }

    /// Returns the world up vector of the given object.
    pub fn up(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let rotation = self.world_rotation(object_id, hierarchy, transforms);
        rotation * Vec3::UP
    }

    /// Returns the world down vector of the given object.
    pub fn down(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Vec3 {
        let rotation = self.world_rotation(object_id, hierarchy, transforms);
        rotation * Vec3::DOWN
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        }
    }
}
