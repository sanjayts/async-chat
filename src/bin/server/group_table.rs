use crate::group::Group;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTable {
    pub fn new() -> GroupTable {
        GroupTable(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, group_name: &String) -> Option<Arc<Group>> {
        self.0.lock().unwrap().get(group_name).cloned()
    }

    pub fn add_and_get(&self, group_name: Arc<String>) -> Arc<Group> {
        self.0
            .lock()
            .unwrap()
            .entry(group_name.clone())
            .or_insert_with(|| Arc::new(Group::new(group_name)))
            .clone()
    }
}
