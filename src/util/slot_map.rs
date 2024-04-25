use nohash_hasher::IntMap;

pub struct SlotMap<T: Sized> {
    data: Vec<T>,
    id_index_map: IntMap<usize, usize>,
    index_id_map: IntMap<usize, usize>,
    free_ids: Vec<usize>,
}

impl<T: Sized> SlotMap<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        self.id_index_map
            .get(&id)
            .and_then(|index| self.data.get(*index))
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.id_index_map
            .get(&id)
            .and_then(|index| self.data.get_mut(*index))
    }

    pub fn allocate(&mut self, item: T) -> usize {
        let id = match self.free_ids.pop() {
            Some(id) => id,
            None => self.data.len(),
        };

        let index = self.data.len();
        self.data.push(item);
        self.id_index_map.insert(id, index);
        self.index_id_map.insert(index, id);

        id
    }

    pub fn deallocate(&mut self, id: usize) {
        let index = if let Some(index) = self.id_index_map.remove(&id) {
            index
        } else {
            return;
        };

        self.data.swap_remove(index);

        let last_index = self.data.len();

        if index != last_index {
            let last_id = *self.index_id_map.get(&last_index).unwrap();
            self.id_index_map.insert(last_id, index);
            self.index_id_map.insert(index, last_id);
        }

        self.id_index_map.remove(&id);
        self.index_id_map.remove(&last_index);
        self.free_ids.push(id);
    }
}

impl<T: Sized> Default for SlotMap<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            id_index_map: IntMap::default(),
            index_id_map: IntMap::default(),
            free_ids: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SlotMap;

    #[test]
    fn test_slotmap() {
        let mut slotmap = SlotMap::new();

        let foo = slotmap.allocate("foo");
        let bar = slotmap.allocate("bar");
        let baz = slotmap.allocate("baz");
        let bazz = slotmap.allocate("bazz");
        let qux = slotmap.allocate("qux");

        assert_eq!(slotmap.get(foo), Some(&"foo"));
        assert_eq!(slotmap.get(bar), Some(&"bar"));
        assert_eq!(slotmap.get(baz), Some(&"baz"));
        assert_eq!(slotmap.get(bazz), Some(&"bazz"));
        assert_eq!(slotmap.get(qux), Some(&"qux"));

        slotmap.deallocate(baz);
        slotmap.deallocate(bazz);

        assert_eq!(slotmap.get(foo), Some(&"foo"));
        assert_eq!(slotmap.get(bar), Some(&"bar"));
        assert_eq!(slotmap.get(qux), Some(&"qux"));

        let quux = slotmap.allocate("quux");

        assert_eq!(slotmap.get(foo), Some(&"foo"));
        assert_eq!(slotmap.get(bar), Some(&"bar"));
        assert_eq!(slotmap.get(qux), Some(&"qux"));
        assert_eq!(slotmap.get(quux), Some(&"quux"));

        slotmap.deallocate(foo);

        assert_eq!(slotmap.get(bar), Some(&"bar"));
        assert_eq!(slotmap.get(qux), Some(&"qux"));
        assert_eq!(slotmap.get(quux), Some(&"quux"));

        let quuux = slotmap.allocate("quuux");

        assert_eq!(slotmap.get(bar), Some(&"bar"));
        assert_eq!(slotmap.get(qux), Some(&"qux"));
        assert_eq!(slotmap.get(quux), Some(&"quux"));
        assert_eq!(slotmap.get(quuux), Some(&"quuux"));
    }
}
