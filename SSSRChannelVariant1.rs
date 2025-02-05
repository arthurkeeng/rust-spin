use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicU8};
use std::sync::atomic::Ordering::{Relaxed , Release , Acquire};


const Empty : u8 = 0 ; 
const Writing : u8 = 1 ; 
const Ready : u8 = 2 ; 
const Reading : u8 = 3; 
pub struct Channel <T>{
    message : UnsafeCell<MaybeUninit<T>>, 
    state: AtomicU8
}

unsafe  impl<T> Sync for Channel<T>{
}

impl<T> Channel<T> {
    pub const fn new()->Self{
        Self{
            message : UnsafeCell::new(MaybeUninit::uninit()),
            state : AtomicU8::new(Empty)
        }
    }

    pub fn send(&self , message : T){
        if self.state.compare_exchange(Empty, Writing, Relaxed, Relaxed).is_err(){
            panic!("Can't send more than one message")
        }
        unsafe{(*self.message.get()).write(message)};
        self.state.store(Ready, Release);
        
    }

    pub fn is_ready(&self) -> bool{
        self.state.load(Relaxed) == Ready
    }
    pub fn receive(&self)->T {
        if self.state.compare_exchange(Ready, Reading, Acquire, Relaxed).is_err(){
            panic!("Message not available")
        }
        unsafe {(*self.message.get()).assume_init_read()}
        
    }
    
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == Ready{
            if *self.state.get_mut() == Ready{
                unsafe {
                    
                    (self.message.get_mut()).assume_init_drop();
                }
            }
        }
    }
    
}
fn main(){}
