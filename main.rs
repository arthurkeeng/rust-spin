use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Release , Acquire};
use std::thread;


// defines the Spinlock struct with a generic parameter
// uses unsafecell and thus can not be passed through thread boundaries
pub struct SpinLock<T>{
  // this uses atomics to keep track of the lock state. 
    locked : AtomicBool, 
    value : UnsafeCell<T>
}

// imitates the Guard common to mutexes;
pub struct Guard<'a , T>{
  // This field contains a reference to the spinlock. We don't want to take ownership of this spinlock 
  // cos the Guard will be dropped at some point to unlock. And we don't want to drop the actual spinlock
  // it also uses a lifetime to indicate that the spinlock the field holds must live longer than the Guard
    lock : &'a SpinLock<T>
}

// this has to implemented as Spinlock contains unsafecell which is not thread safe.
// T is send but doesn't have to be sync because we are not giving references hence it doen't need to fulfil the Sync trait bound
unsafe impl<T> Sync for SpinLock<T> where T : Send {}
impl<T> SpinLock<T>{
  // this constructor helps create a new Spinlock 
    pub const fn new( v : T)->Self{
        Self{
            locked : AtomicBool::new(false), 
            value : UnsafeCell::new(v)
        }
    }

  // This function is responsible for locking the spinlock. It returns Guard which is a wrapper for the SpinLock 
  // this is so we can implement a Drop on it among other reasons
    pub fn lock(&self)-> Guard<T>{
      // here we check if the locked state is false and then we swap the value with true .
      // notice we use the acquire memory ordering here.
        while self.locked.swap(true, Acquire){
            std::hint::spin_loop();
        }
        Guard{
            lock : self
        }
    }

    pub unsafe fn unlock(&self){
        self.locked.store(false, Release);
    }
}


impl<T> DerefMut for Guard<'_ , T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{&mut *self.lock.value.get()}
    }
}

impl<T> Drop for Guard<'_ , T> {
    fn drop(&mut self) {
        unsafe {self.lock.unlock()};
    }
}

impl<T> Deref for Guard<'_ , T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let t = unsafe {&*self.lock.value.get()};
        t
    }
}

fn main(){
    
}
