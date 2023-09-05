extern "C" {
    #[link_name="console_log"]
    fn console_log_ptr(ptr: *const u8, len: i32);

    #[link_name="read"]
    fn read_ptr(ptr: *const u8, len: i32) -> i32;

    #[link_name="write"]
    fn write_ptr(ptr: *const u8, len: i32) -> i32;
}

fn console_log(text: &str) {
    unsafe {
        console_log_ptr(text.as_ptr(), text.len() as i32);
    }
}

fn read(buf: &[u8], len: i32) -> i32 {
    unsafe {
        read_ptr(buf.as_ptr(), len)
    }
}

fn write(buf: &[u8], len: i32) -> i32 {
    unsafe {
        write_ptr(buf.as_ptr(), len)
    }
}

macro_rules! console_log {
    ($($t:tt)*) => {
        console_log(&format_args!($($t)*).to_string().as_str())
    }
}

#[no_mangle]
pub extern fn connect() {
    console_log!("Hello from connection");

    let buf = vec![0; 1024];
    let len = read(&buf, 1);
    console_log!("Read: {}", len);
    if len > 0 {
        console_log!("Text: {}", std::str::from_utf8(&buf[0..len as usize]).unwrap());
        let res = write(&buf, 1);
        console_log!("Write: {}", res);
    }
}
