use std::sync::{Arc, Mutex, MutexGuard, LockResult, PoisonError};
use lazy_static::lazy_static;
use std::ops::{Deref, DerefMut};

struct MutexWrap<T: ?Sized>(Mutex<T>);

impl<T> MutexWrap<T> {
    fn new(t: T) -> Self {
        MutexWrap(Mutex::new(t))
    }

    fn lock(&self) -> LockResult<MutexGuardWrap<T>> {
        println!("Locking");
        match self.0.lock() {
            Ok(guard) => Ok(MutexGuardWrap(guard)),
            Err(poison) => Err(PoisonError::new(MutexGuardWrap(poison.into_inner())))
        }
    }
}

struct MutexGuardWrap<'a, T: ?Sized + 'a>(MutexGuard<'a, T>);

impl<'a, T: ?Sized + 'a> Deref for MutexGuardWrap<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: ?Sized + 'a> DerefMut for MutexGuardWrap<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: ?Sized + 'a> Drop for MutexGuardWrap<'a, T> {
    fn drop(&mut self) {
        println!("Unlocking");
    }
}

struct Resource {
    count: Arc<()>,
}

impl Clone for Resource {
    fn clone(&self) -> Self {
        println!("Cloning, refcount: {}", Arc::strong_count(&self.count));
        Resource {
            count: Arc::clone(&self.count),
        }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        println!("Dropping, refcount: {}", Arc::strong_count(&self.count));
        // If the strong count is 2, the only other place where this exists is the cache
        if Arc::strong_count(&self.count) == 2 {
            let mut cache = CACHE.lock().unwrap();
            if let Some(_) = cache.get_resource() { // no deadlock
            //if let Some(_l) = cache.get_resource() { // deadlock after drop from cache
            //if cache.get_resource().is_some() { // deadlock during drop from cache
                println!("Dropping from the cache");
                cache.drop_resource();
                println!("Dropped from the cache");
            }
            // BONUS!
            // Adding a statement here, even if it doesn't use `cache`,
            // deadlocks even with Some(_)
            // println!("Get rekt");
        }
    }
}

struct Cache {
    resource: Option<Resource>,
}

impl Cache {
    /// A method that exists just to experiment with when the cache gets dropped
    fn nothing(&mut self) {}

    fn add_resource(&mut self, res: &Resource) {
        self.resource = Some(Resource::clone(res));
    }

    fn get_resource(&self) -> Option<Resource> {
        self.resource.as_ref().map(|res| Resource::clone(&res))
    }

    fn drop_resource(&mut self) {
        self.resource = None;
    }
}

lazy_static! {
    static ref CACHE: MutexWrap<Cache> = MutexWrap::new(Cache { resource: None });
}

fn main() {
    {
        let res = Resource {
            count: Arc::new(()),
        };

        CACHE.lock().unwrap().add_resource(&res);
    }
}
