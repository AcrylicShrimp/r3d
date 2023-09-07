use super::ObjectId;
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct ObjectIdAllocator {
    next_id: NonZeroU32,
    freed_ids: Vec<ObjectId>,
}

impl ObjectIdAllocator {
    pub fn new() -> Self {
        Self {
            next_id: NonZeroU32::new(1).unwrap(),
            freed_ids: Vec::with_capacity(1024),
        }
    }

    pub fn alloc(&mut self) -> ObjectId {
        match self.freed_ids.pop() {
            Some(id) => id,
            None => {
                let id = ObjectId::new(self.next_id);
                self.next_id = self
                    .next_id
                    .checked_add(1)
                    .unwrap_or_else(|| NonZeroU32::new(1).unwrap());
                id
            }
        }
    }

    pub fn dealloc(&mut self, id: ObjectId) {
        self.freed_ids.push(id);
    }
}
