pub mod context;
pub mod observable;

#[cfg(test)]
mod test {
    use super::context::Runtime;

    #[test]
    fn basic() {
        let runtime = Box::leak(Box::new(Runtime::default()));
        let a = runtime.create_obseravble(0);
        assert_eq!(a.get(), 0);
        a.set(5);
        assert_eq!(a.get(), 5);
    }

    #[test]
    fn derived() {
        let runtime = Box::leak(Box::new(Runtime::default()));
        let a = runtime.create_obseravble(0);
        let b = runtime.create_obseravble(0);
        let c = move || a.get() + b.get();
        assert_eq!(c(), 0);
        a.set(5);
        assert_eq!(c(), 5);
        b.set(3);
        assert_eq!(c(), 8);
    }
}
