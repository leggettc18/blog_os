// main.rs

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main =  "test_main"]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use blog_os::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
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

  // trigger a page fault
  // let ptr = 0xdeadbeaf as *mut u32;
  // unsafe { *ptr = 42; }

  use x86_64::registers::control::Cr3;

  let (level_4_page_table, _) = Cr3::read();
  println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

  #[cfg(test)]
  test_main();

  println!("It did not crash!");
  blog_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  blog_os::hlt_loop();
}

/// This function is called on panic while in test mode
/// i.e. when a test fails.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  blog_os::test_panic_handler(info)
}
