extern crate hyper;
use std::io::Read;
use hyper::Client;
use hyper::header::Connection;
use std::ffi::CStr;

struct HTTP_Data {
    data : Option<Vec<u8>>,
    async : bool,
    successful : bool
}

#[no_mangle]
pub extern fn http_get(url : *const i8) -> *mut Vec<u8> {
    let url = unsafe {
        CStr::from_ptr(url).to_str().unwrap().to_owned()
    };

    let client = Client::new();
    let mut response = match client.get(&url).header(Connection::close()).send() {
        Ok(n) => n,
        Err(e) => {
            println!("Error retrieving {}.", url);
            println!("The error was: {}", e);
            return 0 as *mut Vec<u8>;
        }
    };

    let mut body_data = Vec::new();
    match response.read_to_end(&mut body_data) {
        Ok(_) => (),
        Err(e) => {
            println!("Error reading {}.", url);
            println!("The error was: {}", e);
            return 0 as *mut Vec<u8>;
        }
    };

    body_data.push(0);
    Box::into_raw(Box::new(body_data))
}

#[no_mangle]
pub unsafe extern fn http_destroy_response(string : *mut Vec<u8>) {
    if string != 0 as *mut _ {
        Box::from_raw(string);
    }
}

#[no_mangle]
pub unsafe extern fn http_response_is_null(string : *const Vec<u8>) -> bool {
    return string == 0 as *mut _
}

#[no_mangle]
pub unsafe extern fn http_read_response(string : *const Vec<u8>) -> *const u8 {
    return (*string).as_ptr()
}

#[no_mangle]
pub unsafe extern fn http_response_length(string : *const Vec<u8>) -> u32 {
    if !http_response_is_null(string) {
        (*string).len() as u32 - 1
    }
    else {
        0
    }
}
