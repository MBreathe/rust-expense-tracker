use crate::models::Expense;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    store: Arc<Mutex<HashMap<Uuid, Expense>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn list(&self) -> Vec<Expense> {
        let store = self.store.lock().unwrap();
        store.values().cloned().collect()
    }
    pub fn insert(&self, expense: Expense) {
        let mut store = self.store.lock().unwrap();
        store.insert(expense.id, expense);
    }
    pub fn get(&self, id: Uuid) -> Option<Expense> {
        let store = self.store.lock().unwrap();
        store.get(&id).cloned()
    }
    pub fn remove(&self, id: Uuid) -> Option<Expense> {
        let mut store = self.store.lock().unwrap();
        store.remove(&id)
    }
}
