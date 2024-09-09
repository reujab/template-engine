use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::Arc,
};

use crate::value::OwnedValue;

pub trait Variables {
    fn get(&self, identifier: &str) -> Option<&OwnedValue>;
}

impl Variables for HashMap<String, OwnedValue> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier)
    }
}

impl Variables for HashMap<String, &OwnedValue> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).copied()
    }
}

impl Variables for HashMap<String, Rc<OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &**v)
    }
}

impl Variables for HashMap<String, Rc<&OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &***v)
    }
}

impl Variables for HashMap<String, Arc<OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &**v)
    }
}

impl Variables for HashMap<String, Arc<&OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &***v)
    }
}

impl Variables for BTreeMap<String, OwnedValue> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier)
    }
}

impl Variables for BTreeMap<String, &OwnedValue> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).copied()
    }
}

impl Variables for BTreeMap<String, Rc<OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &**v)
    }
}

impl Variables for BTreeMap<String, Rc<&OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &***v)
    }
}

impl Variables for BTreeMap<String, Arc<OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &**v)
    }
}

impl Variables for BTreeMap<String, Arc<&OwnedValue>> {
    fn get(&self, identifier: &str) -> Option<&OwnedValue> {
        self.get(identifier).map(|v| &***v)
    }
}
