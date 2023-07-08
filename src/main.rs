#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lite_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lite_os::println;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    lite_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lite_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    lite_os::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    lite_os::hlt_loop();
}