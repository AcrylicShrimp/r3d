use super::ObjectId;
use std::collections::{HashMap, HashSet};

pub struct ObjectNameRegistry {
    object_names: HashMap<ObjectId, String>,
    object_ids: HashMap<String, HashSet<ObjectId>>,
}

impl ObjectNameRegistry {
    pub fn new() -> Self {
        Self {
            object_names: HashMap::new(),
            object_ids: HashMap::new(),
        }
    }

    pub fn name(&self, object: ObjectId) -> Option<&String> {
        self.object_names.get(&object)
    }

    pub fn ids<'a>(&'a self, name: &str) -> Option<impl Iterator<Item = ObjectId> + 'a> {
        self.object_ids.get(name).map(|ids| ids.iter().copied())
    }

    pub fn set_name(&mut self, object: ObjectId, name: Option<String>) {
        self.decouple(object);

        if let Some(name) = name {
            self.object_names.insert(object, name.clone());
            self.object_ids
                .entry(name)
                .or_insert_with(HashSet::new)
                .insert(object);
        }
    }

    fn decouple(&mut self, object: ObjectId) {
        if let Some(name) = self.object_names.remove(&object) {
            if let Some(object_ids) = self.object_ids.get_mut(&name) {
                object_ids.remove(&object);
            }
        }
    }
}
