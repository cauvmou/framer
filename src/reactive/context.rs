use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashSet,
};

use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::observable::{Observable, ObservableId};

#[derive(Default)]
pub struct Runtime {
    pub(crate) observable_values: RefCell<SlotMap<ObservableId, Box<RefCell<dyn Any>>>>,
    pub(crate) running_observer: Cell<Option<ObserverId>>,
    pub(crate) subscribed_observers: RefCell<SecondaryMap<ObservableId, HashSet<ObserverId>>>,
    pub(crate) observers: RefCell<SlotMap<ObserverId, Box<dyn Fn()>>>,
}

impl Runtime {
    pub fn create_obseravble<T: 'static>(&'static self, value: T) -> Observable<T> {
        let id = self
            .observable_values
            .borrow_mut()
            .insert(Box::new(RefCell::new(value)));
        Observable {
            cx: self,
            id,
            ty: std::marker::PhantomData,
        }
    }

    pub fn create_observer(&'static self, observer: impl Fn() + 'static) {
        self.run_observer(self.observers.borrow_mut().insert(Box::new(observer)))
    }

    pub fn run_observer(&self, observer_id: ObserverId) {
        let prev = self.running_observer.take();
        self.running_observer.set(Some(observer_id));

        let binding = self.observers.borrow();
        let observer = binding.get(observer_id).unwrap();
        observer();

        self.running_observer.set(prev);
    }
}

new_key_type! {
    pub struct ObserverId;
}
