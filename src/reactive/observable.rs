use std::marker::PhantomData;

use slotmap::new_key_type;

use super::context::Runtime;

#[derive(Clone, Copy)]
pub struct Observable<T: 'static> {
    pub(crate) cx: &'static Runtime,
    pub(crate) id: ObservableId,
    pub(crate) ty: PhantomData<T>,
}

impl<T> Observable<T> {
    pub fn with(&self, with: impl Fn(&T) + 'static) {
        let binding = self.cx.observable_values.borrow();
        let value = binding.get(self.id).unwrap();
        let value = value.borrow();
        self.add_subscriber();
        with(value.downcast_ref::<T>().unwrap());
    }

    pub fn set(&self, value: T) {
        {
            let binding = self.cx.observable_values.borrow();
            let wrapper = binding.get(self.id).unwrap();
            let mut wrapper = wrapper.borrow_mut();
            let wrapper = wrapper.downcast_mut::<T>().unwrap();
            *wrapper = value;
        }
        self.notify_observers();
    }

    fn notify_observers(&self) {
        if let Some(subs) = self.cx.subscribed_observers.borrow().get(self.id).cloned() {
            for sub in subs {
                self.cx.run_observer(sub);
            }
        }
    }

    fn add_subscriber(&self) {
        if let Some(running_observer) = self.cx.running_observer.get() {
            let mut subs = self.cx.subscribed_observers.borrow_mut();
            let subs = subs.entry(self.id).unwrap().or_default();
            subs.insert(running_observer);
        }
    }
}

impl<T: Clone> Observable<T> {
    pub fn get(&self) -> T {
        let binding = self.cx.observable_values.borrow();
        let value = binding.get(self.id).unwrap();
        let value = value.borrow();
        self.add_subscriber();
        value.downcast_ref::<T>().unwrap().clone()
    }
}

new_key_type! {
    pub struct ObservableId;
}
