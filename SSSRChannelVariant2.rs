

use std::marker::PhantomData;
use std::thread::{self, Thread};
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};
use std::sync::atomic::Ordering::{Relaxed , Release , Acquire};


pub struct Channel<T>{
    message : UnsafeCell<MaybeUninit<T>>,
    ready : AtomicBool
}

unsafe impl<T> Sync for Channel<T> where T : Send{}
impl<T> Channel<T> {
    pub fn new()-> Self{
        Self { message: 
            UnsafeCell::new(MaybeUninit::uninit()), ready: AtomicBool::new(false) }
    }

    pub fn split(&mut self) -> (Sender< T> , Receiver<T>){
         *self = Self::new();
         (Sender{
            channel : self, 
            receiving_thread : thread::current()
         } , Receiver {
            channel : self , 
            _no_send : PhantomData
         })
    }
}
pub struct Sender<'a , T>{
    channel : &'a Channel<T> , 
    receiving_thread : Thread
}

impl<T> Sender<'_ , T>{
    pub fn is_ready(&self)-> bool{
        self.channel.ready.load(Relaxed)
    }
    pub fn send(self , message : T){
       unsafe{ (*self.channel.message.get()).write(message)};
       self.channel.ready.store(true , Release);
       self.receiving_thread.unpark();

    }
}
pub struct Receiver<'a , T>{
    channel : &'a Channel<T> , 
    _no_send : PhantomData<*const ()>
}

impl<T> Receiver<'_ , T> {
    pub fn receive(&self) -> T{
        if !self.channel.ready.swap(true , Acquire){
            thread::park();
        }
        unsafe{(*self.channel.message.get()).assume_init_read()}
    }
    
}
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut(){

            unsafe { self.message.get_mut().assume_init_drop();}
        }
    }
    
}
fn main(){

   
}
