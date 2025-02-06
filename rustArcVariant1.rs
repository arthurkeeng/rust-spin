use std::os::windows::process;
use std::sync::atomic::fence;
use std::{cell::UnsafeCell, ops::Deref, ptr::NonNull, sync::atomic::AtomicUsize};
use std::sync::atomic::Ordering::{Relaxed , Release , Acquire};


struct ArcData<T>{
    data_ref_count : AtomicUsize, 
    alloc_ref_count : AtomicUsize, 
    data : UnsafeCell<Option<T>>
}
pub struct Arc<T>{
    weak : Weak<T>
}
unsafe impl<T> Sync for Weak<T> where T : Send +Sync{}
unsafe impl<T> Send for Weak<T> where T : Send +Sync{}
pub struct Weak<T>{
    ptr : NonNull<ArcData<T>>
}
impl<T> Arc<T>{
    pub fn new(data : T) ->Self{
        Self { weak: 
        Weak{
            ptr: NonNull::from(Box::leak(Box::new(
                ArcData{
                    data_ref_count : AtomicUsize::new(1),
                    alloc_ref_count : AtomicUsize::new(1), 
                    data : UnsafeCell::new(Some(data))
                }
            )))
        } }
        
    }
pub fn downgrade(arc:&Self) -> Weak<T>{
    arc.weak.clone()
        }
    pub fn get_mut(arc : &mut Self) -> Option<&mut T>{
        if arc.weak.data().alloc_ref_count.load(Relaxed) == 1{
            fence(Acquire);
            let a = Some(unsafe{(*arc.weak.ptr.as_mut().data.get_mut()).as_mut().unwrap()});
            a
        }
        else {
            None
        }
    }
 
}

impl<T> Weak<T>{
    fn data(&self) -> &ArcData<T>{
        unsafe {
            
            self.ptr.as_ref()
        }
    }
    pub fn upgrade(&self) -> Option<Arc<T>>{
        let mut n = self.data().data_ref_count.load(Relaxed);
        loop{

            if n == 0 {
                return None ; 
            }
            assert!(n < usize::MAX);
            if let Err(e) = 
                self.data().data_ref_count.compare_exchange(n, n + 1, Relaxed, Relaxed){
                    n = e;
                    continue;
                }
                return Some(Arc{
                    weak : self.clone()
                })
        }
    }

   
}

impl<T> Deref for Arc<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let ptr = self.weak.data().data.get();

        let a = unsafe {
            
            (*ptr).as_ref().unwrap()
        };
        a
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Relaxed) > usize::MAX /2{
            std::process::abort();
        };
        Self { ptr: 
        self.ptr
         }
    }
}


impl<T> Clone for Arc<T>  {
    
    fn clone(&self) -> Self {
        let weak = self.weak.clone();
        if self.weak.data().data_ref_count.fetch_add(1, Relaxed) > usize::MAX /2{
            std::process::abort();
        }
        Self{
            weak
        }
    }
    
}

impl<T> Drop for Weak<T>{
    fn drop(&mut self){
        if self.data().alloc_ref_count.fetch_sub(1, Relaxed) == 1{
            fence(Acquire);
            unsafe{Box::from_raw(self.ptr.as_mut())};
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
      if  self.weak.data().data_ref_count.fetch_sub(1, Release) == 1{
        let ptr = self.weak.data().data.get();

        unsafe{
            (*ptr) = None
        }
      }
    }
    
}

fn  main(){}
