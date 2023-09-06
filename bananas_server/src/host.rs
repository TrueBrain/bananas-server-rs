extern "C" {
    #[link_name = "console_log"]
    fn console_log_ptr(ptr: *const u8, len: i32);

    #[link_name = "read"]
    fn read_ptr(ptr: *const u8, len: i32) -> i32;

    #[link_name = "write"]
    fn write_ptr(ptr: *const u8, len: i32) -> i32;
}

pub fn console_log(text: &str) {
    unsafe {
        console_log_ptr(text.as_ptr(), text.len() as i32);
    }
}

pub fn read(buf: &[u8], len: i32) -> Result<i32, i32> {
    let res = unsafe { read_ptr(buf.as_ptr(), len) };
    if res < 0 {
        Err(res)
    } else {
        Ok(res)
    }
}

pub fn write(buf: &[u8], len: i32) -> Result<i32, i32> {
    let res = unsafe { write_ptr(buf.as_ptr(), len) };
    if res < 0 {
        Err(res)
    } else {
        Ok(res)
    }
}

macro_rules! console_log {
    ($($t:tt)*) => {
        console_log(&format_args!($($t)*).to_string().as_str())
    }
}
