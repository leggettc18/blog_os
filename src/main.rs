// main.rs

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main =  "test_main"]

use core::panic::PanicInfo;
use blog_os::println;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
  // this function is the entry point, since the linker looks for a function
  // named '_start' by default
  println!("Hello World{}", "!");

  // init function from lib.rs
  blog_os::init();

  // invoke a breakpoint exception
  x86_64::instructions::interrupts::int3();

  // trigger a page fault
  // unsafe {
  //   *(0xdeadbeef as *mut u64) = 42;
  // };

  // recurses endlessly to cause a stack overflow
  // fn stack_overflow() {
  //   stack_overflow();
  // }

  // trigger a stack overflow
  // stack_overflow();

  #[cfg(test)]
  test_main();

  println!("It did not crash!");
  loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {}
}

/// This function is called on panic while in test mode
/// i.e. when a test fails.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  blog_os::test_panic_handler(info)
}
