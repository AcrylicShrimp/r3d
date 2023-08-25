use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(NonZeroU32);

impl ObjectId {
    pub fn new(id: NonZeroU32) -> Self {
        Self(id)
    }

    pub fn from_u32(id: u32) -> Self {
        Self(NonZeroU32::new(id + 1).unwrap())
    }

    pub fn get(&self) -> u32 {
        self.0.get() - 1
    }
}

impl From<ObjectId> for u32 {
    fn from(id: ObjectId) -> Self {
        id.0.get() - 1
    }
}
