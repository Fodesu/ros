#![no_std]
#![no_main]
use core::panic::PanicInfo;


#[panic_handler]
fn panic(_info : &PanicInfo) ->! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! { // ! 表示该函数没有返回
    loop {}
}