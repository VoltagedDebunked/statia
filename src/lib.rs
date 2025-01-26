use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::any::Any;

// Core state container
#[derive(Clone)]
pub struct State<T> {
    inner: Arc<RwLock<T>>,
    subscribers: Arc<Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
}

// State manager that can hold multiple states
pub struct StateManager {
    states: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl<T: Clone + Send + Sync + 'static> State<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        self.inner.read().unwrap().clone()
    }

    pub fn set(&self, new_value: T) {
        let mut inner = self.inner.write().unwrap();
        *inner = new_value;
        drop(inner);
        
        // Notify subscribers
        let subscribers = self.subscribers.lock().unwrap();
        let current = self.get();
        for subscriber in subscribers.iter() {
            subscriber(&current);
        }
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(Box::new(callback));
    }

    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut inner = self.inner.write().unwrap();
        updater(&mut inner);
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<T: Clone + Send + Sync + 'static>(&self, key: &str, initial: T) -> State<T> {
        let state = State::new(initial);
        let mut states = self.states.write().unwrap();
        states.insert(key.to_string(), Box::new(state.clone()));
        state
    }

    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<State<T>> {
        let states = self.states.read().unwrap();
        states.get(key)
            .and_then(|boxed| boxed.downcast_ref::<State<T>>())
            .cloned()
    }
}

// Transaction support for atomic updates
pub struct Transaction<T> {
    state: State<T>,
    operations: Vec<Box<dyn FnOnce(&mut T)>>,
}

impl<T: Clone + Send + Sync + 'static> Transaction<T> {
    pub fn new(state: State<T>) -> Self {
        Self {
            state,
            operations: Vec::new(),
        }
    }

    pub fn update<F>(&mut self, operation: F)
    where
        F: FnOnce(&mut T) + 'static,
    {
        self.operations.push(Box::new(operation));
    }

    pub fn commit(self) {
        self.state.update(|value| {
            for op in self.operations {
                op(value);
            }
        });
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_state() {
        let state = State::new(0);
        assert_eq!(state.get(), 0);
        
        state.set(42);
        assert_eq!(state.get(), 42);
    }

    #[test]
    fn test_state_manager() {
        let manager = StateManager::new();
        let count_state = manager.register("count", 0);

        count_state.set(10);
        assert_eq!(count_state.get(), 10);
        
        let retrieved_count = manager.get::<i32>("count").unwrap();
        assert_eq!(retrieved_count.get(), 10);
    }

    #[test]
    fn test_transaction() {
        let state = State::new(vec![1, 2, 3]);
        let mut transaction = Transaction::new(state.clone());
        
        transaction.update(|v| v.push(4));
        transaction.update(|v| v.push(5));
        
        assert_eq!(state.get(), vec![1, 2, 3]);
        transaction.commit();
        assert_eq!(state.get(), vec![1, 2, 3, 4, 5]);
    }
}