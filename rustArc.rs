use std::{ops::Deref, ptr::NonNull, sync::atomic::AtomicUsize, thread};

use std::sync::atomic::Ordering::{Relaxed , Acquire , Release};
use std::sync::atomic::fence;


struct ArcData<T>{
    ref_count : AtomicUsize, 
    data : T
}
pub struct Arc<T>{
    ptr: NonNull<ArcData<T>>
}

impl<T> Arc<T> {

    pub fn new(data : T) -> Arc<T>{
        Arc{
            ptr : 
            NonNull::from(Box::leak(Box::new(ArcData{
                ref_count : AtomicUsize::new(0),
                data
            })))
        }

    }
    fn data(&self) -> &ArcData<T>{
        unsafe {
            
            self.ptr.as_ref()
        }
    }
    pub fn get_mut(arc : &mut Self)-> Option<&mut T>{
        if arc.data().ref_count.load(Relaxed) == 1{
            fence(Acquire);
           Some( unsafe{&mut arc.ptr.as_mut().data})
        }
        else {
            None
        }
    }
    
}

impl<T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data().data
    }
    
}

impl<T> Clone for Arc<T>{
    fn clone(&self) -> Self {
        if self.data().ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        };
        Arc { ptr: self.ptr }
    }
}


impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe{drop(Box::from_raw(self.ptr.as_ptr()))};
        }
    }
    
}

unsafe impl <T> Sync for Arc<T> where T : Send + Sync{  
}
unsafe impl <T> Send for Arc<T> where T : Send + Sync{  
}
fn main(){
    let a = AtomicUsize::new(3);

    let b = a.fetch_add(1, Relaxed);

    println!("{b:?}")

 }

