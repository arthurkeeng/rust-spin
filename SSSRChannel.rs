use std::thread;
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};
use std::sync::atomic::Ordering::{Relaxed , Acquire , Release};


// this is the structure of the basic channel
pub struct Channel<T> {
    in_use : AtomicBool , 
    message : UnsafeCell<MaybeUninit<T>> , 
    ready : AtomicBool
}
// we need to impl sync for channel as it contains an unsafe cell;
unsafe impl<T> Sync for Channel<T> where T : Send{}
impl<T> Channel<T> {
  // constructor to create a new Channel. 
    pub const fn new()-> Self{
        Self{
          // this field checks to make sure only one sender is currently working
          
            in_use : AtomicBool::new(false), 
          // the message to be sent . This uses a mybeUninit to set the initial state to uninitialized
            message : UnsafeCell::new(MaybeUninit::uninit()), 
          // this helps us know when we have or do not have a message
            ready : AtomicBool::new(false)
        }
    }
  // this helps in sending the message
    pub fn send(&self , message : T) {
      // swap returns the initial value before the swap. Hence if the value was true , we panic as we only want
      // one sender to work at a time
        if self.in_use.swap(true , Relaxed){
            panic!("One message in pipeline")
        }
      // the maybeUninit has a write function that replaces the current value with some other value
         unsafe {(*self.message.get()).write(message)};
      // we set the ready flag to true and we use the release-store ordering for that
         self.ready.store(true, Release);


    }
  // a function for retrieving the ready state of the channel . Nothing too fancy
    pub fn is_ready(&self) -> bool{
        self.ready.load(Relaxed)
    }

  // function to receive the sent value
    pub  fn receive(&self)-> T{
      // recall that swap returns the initial value it found while performing a swap . 
      // if the receive was called when no message has been sent (ready = false )
      // this receive function panics
        if !self.ready.swap(true , Acquire){
            panic!("No message available");
        }
      // if there is a message , we get the message. Recall that message is behind a maybeUninit which has 
      // a helper function that assumes the value has been initialized and returns the value to us
       unsafe{ (*self.message.get()).assume_init_read()}
    }
    
}

// we implement drop for the message for 
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        let a = self.message.get_mut();
        if *self.ready.get_mut(){
           unsafe { self.message.get_mut().assume_init_drop()};
        }
    }
    
}
fn main(){
    let channel = Channel::new();
    let t = thread::current();

    thread::scope(|s|{
        s.spawn(||{
            channel.send("hello there");
            t.unpark();
        });
        while !channel.is_ready(){

            thread::park();
        }

    });
    let message = channel.receive();

    println!("{message}")
}
