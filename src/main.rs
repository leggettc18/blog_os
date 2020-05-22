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
  use blog_os::memory;
  use blog_os::memory::BootInfoFrameAllocator;
  use x86_64::{structures::paging::Page, VirtAddr};

  println!("Hello World{}", "!");

  // init function from lib.rs
  blog_os::init();

  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  let mut mapper = unsafe { memory::init(phys_mem_offset) };
  let mut frame_allocator = unsafe {
    BootInfoFrameAllocator::init(&boot_info.memory_map)
  };

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
