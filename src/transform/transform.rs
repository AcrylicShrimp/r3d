use crate::{
    math::{Mat4, Quat, Vec3, Vec4},
    object::{ObjectComponent, ObjectHandle, ObjectHierarchy, ObjectId},
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
    /// This matrix does not include the parent transforms.
    pub fn matrix(&self) -> Mat4 {
        Mat4::srt(self.position, self.rotation, self.scale)
    }

    /// Returns the inverse transform matrix that transforms from world space to local space.
    /// This matrix does not include the parent transforms.
    pub fn inverse_matrix(&self) -> Mat4 {
        Mat4::trs(-self.position, -self.rotation, Vec3::recip(self.scale))
    }

    /// Returns the transform matrix that transforms from local space to world space.
    /// This matrix includes the parent transforms.
    pub fn world_matrix(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Mat4 {
        let mut matrix = self.matrix();

        for &parent_id in hierarchy.parents(object_id) {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                matrix *= parent_transform.matrix();
            }
        }

        matrix
    }

    /// Returns the inverse transform matrix that transforms from world space to local space.
    /// This matrix includes the parent transforms.
    pub fn world_inverse_matrix(
        &self,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &ReadStorage<Transform>,
    ) -> Mat4 {
        let mut matrix = Mat4::identity();

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                matrix *= parent_transform.inverse_matrix();
            }
        }

        matrix * self.inverse_matrix()
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
        position: Vec3,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &mut WriteStorage<Transform>,
    ) {
        let mut position = Vec4::from_vec3(position, 1.0);

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                position *= parent_transform.inverse_matrix();
            }
        }

        transforms
            .get_mut(hierarchy.entity(object_id))
            .unwrap()
            .position = position.into();
    }

    /// Sets the world rotation of the given object.
    pub fn set_world_rotation(
        rotation: Quat,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &mut WriteStorage<Transform>,
    ) {
        let mut rotation = rotation;

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                rotation *= parent_transform.rotation.conjugated();
            }
        }

        transforms
            .get_mut(hierarchy.entity(object_id))
            .unwrap()
            .rotation = rotation;
    }

    /// Sets the world scale of the given object.
    pub fn set_world_scale(
        scale: Vec3,
        object_id: ObjectId,
        hierarchy: &ObjectHierarchy,
        transforms: &mut WriteStorage<Transform>,
    ) {
        let mut scale = scale;

        for &parent_id in hierarchy.parents(object_id).iter().rev() {
            let parent_entity = hierarchy.entity(parent_id);
            if let Some(parent_transform) = transforms.get(parent_entity) {
                scale /= parent_transform.scale;
            }
        }

        transforms
            .get_mut(hierarchy.entity(object_id))
            .unwrap()
            .scale = scale;
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

pub struct TransformComponent {
    object: ObjectHandle,
}

impl ObjectComponent for TransformComponent {
    type Component = Transform;

    fn new(object: ObjectHandle) -> Self {
        Self { object }
    }

    fn object(&self) -> &ObjectHandle {
        &self.object
    }
}

impl TransformComponent {
    /// Returns the transform matrix that transforms from local space to world space.
    /// This matrix does not include the parent transforms.
    pub fn matrix(&self) -> Mat4 {
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().matrix()
    }

    /// Returns the inverse transform matrix that transforms from world space to local space.
    /// This matrix does not include the parent transforms.
    pub fn inverse_matrix(&self) -> Mat4 {
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().inverse_matrix()
    }

    /// Returns the transform matrix that transforms from local space to world space.
    /// This matrix includes the parent transforms.
    pub fn world_matrix(&self) -> Mat4 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .world_matrix(object_id, &hierarchy, &transforms)
    }

    /// Returns the inverse transform matrix that transforms from world space to local space.
    /// This matrix includes the parent transforms.
    pub fn world_inverse_matrix(&self) -> Mat4 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .world_inverse_matrix(object_id, &hierarchy, &transforms)
    }

    /// Returns the local position of the given object.
    pub fn position(&self) -> Vec3 {
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().position
    }

    /// Returns the local rotation of the given object.
    pub fn rotation(&self) -> Quat {
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().rotation
    }

    /// Returns the local scale of the given object.
    pub fn scale(&self) -> Vec3 {
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().scale
    }

    /// Sets the local position of the given object.
    pub fn set_position(&self, position: Vec3) {
        let mut object_mgr = self.object.ctx.object_mgr_mut();
        object_mgr
            .object_hierarchy_mut()
            .set_dirty(self.object.object_id);

        let world = self.object.ctx.world();
        let mut transforms = world.write_component::<Transform>();
        transforms.get_mut(self.object.entity).unwrap().position = position;
    }

    /// Sets the local rotation of the given object.
    pub fn set_rotation(&self, rotation: Quat) {
        let mut object_mgr = self.object.ctx.object_mgr_mut();
        object_mgr
            .object_hierarchy_mut()
            .set_dirty(self.object.object_id);

        let world = self.object.ctx.world();
        let mut transforms = world.write_component::<Transform>();
        transforms.get_mut(self.object.entity).unwrap().rotation = rotation;
    }

    /// Sets the local scale of the given object.
    pub fn set_scale(&self, scale: Vec3) {
        let mut object_mgr = self.object.ctx.object_mgr_mut();
        object_mgr
            .object_hierarchy_mut()
            .set_dirty(self.object.object_id);

        let world = self.object.ctx.world();
        let mut transforms = world.write_component::<Transform>();
        transforms.get_mut(self.object.entity).unwrap().scale = scale;
    }

    /// Returns the world position of the given object.
    pub fn world_position(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().world_position(
            object_id,
            &hierarchy,
            &transforms,
        )
    }

    /// Returns the world rotation of the given object.
    pub fn world_rotation(&self) -> Quat {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms.get(self.object.entity).unwrap().world_rotation(
            object_id,
            &hierarchy,
            &transforms,
        )
    }

    /// Returns the world scale of the given object.
    pub fn world_scale(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .world_scale(object_id, &hierarchy, &transforms)
    }

    /// Sets the world position of the given object.
    pub fn set_world_position(&self, position: Vec3) {
        let object_id = self.object.object_id;
        let mut object_mgr = self.object.ctx.object_mgr_mut();
        let hierarchy = object_mgr.object_hierarchy_mut();
        hierarchy.set_dirty(self.object.object_id);

        let world = self.object.ctx.world();
        let mut transforms = world.write_component::<Transform>();
        Transform::set_world_position(position, object_id, &hierarchy, &mut transforms);
    }

    /// Sets the world rotation of the given object.
    pub fn set_world_rotation(&self, rotation: Quat) {
        let object_id = self.object.object_id;
        let mut object_mgr = self.object.ctx.object_mgr_mut();
        let hierarchy = object_mgr.object_hierarchy_mut();
        hierarchy.set_dirty(self.object.object_id);

        let world = self.object.ctx.world();
        let mut transforms = world.write_component::<Transform>();
        Transform::set_world_rotation(rotation, object_id, &hierarchy, &mut transforms);
    }

    /// Sets the world scale of the given object.
    pub fn set_world_scale(&self, scale: Vec3) {
        let object_id: ObjectId = self.object.object_id;
        let mut object_mgr = self.object.ctx.object_mgr_mut();
        let hierarchy = object_mgr.object_hierarchy_mut();
        hierarchy.set_dirty(self.object.object_id);

        let world = self.object.ctx.world();
        let mut transforms = world.write_component::<Transform>();
        Transform::set_world_scale(scale, object_id, &hierarchy, &mut transforms);
    }

    /// Returns the world forward vector of the given object.
    pub fn forward(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .forward(object_id, &hierarchy, &transforms)
    }

    /// Returns the world right vector of the given object.
    pub fn backward(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .backward(object_id, &hierarchy, &transforms)
    }

    /// Returns the world right vector of the given object.
    pub fn right(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .right(object_id, &hierarchy, &transforms)
    }

    /// Returns the world left vector of the given object.
    pub fn left(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .left(object_id, &hierarchy, &transforms)
    }

    /// Returns the world up vector of the given object.
    pub fn up(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .up(object_id, &hierarchy, &transforms)
    }

    /// Returns the world down vector of the given object.
    pub fn down(&self) -> Vec3 {
        let object_id = self.object.object_id;
        let object_mgr = self.object.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();
        let world = self.object.ctx.world();
        let transforms = world.read_component::<Transform>();
        transforms
            .get(self.object.entity)
            .unwrap()
            .down(object_id, &hierarchy, &transforms)
    }
}
