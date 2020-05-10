#![no_std]
#![no_main]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World";
// 这个函数将在panic时调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// 使用`no_mangle`禁用名称重整, 确保rust编译器
/// 输出的是名为`_start`的函数
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}
