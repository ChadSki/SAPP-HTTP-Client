extern crate hyper;
use std::sync::{Arc, Mutex};
use std::io::Read;
use hyper::Client;
use hyper::header::Connection;
use std::ffi::CStr;

pub struct HttpResponse {
    pub data : Arc<Mutex<Option<Vec<u8>>>>,
    successful : Arc<Mutex<bool>>,
    complete : Arc<Mutex<bool>>
}

impl HttpResponse {
    fn new() -> HttpResponse {
        HttpResponse {
            data : Arc::new(Mutex::new(None)),
            successful : Arc::new(Mutex::new(false)),
            complete : Arc::new(Mutex::new(false))
        }
    }
    fn is_complete(&self) -> bool {
        match self.complete.try_lock() {
            Ok(n) => *n,
            Err(_) => {
                false
            }
        }
    }
    fn is_successful(&self) -> bool {
        match self.successful.try_lock() {
            Ok(n) => *n,
            Err(_) => {
                false
            }
        }
    }
}

#[no_mangle]
pub unsafe extern fn http_get(url_ptr : *const i8, async : bool) -> *mut HttpResponse {
    let url = CStr::from_ptr(url_ptr).to_str().unwrap().to_owned();

    let response = HttpResponse::new();
    let successful = response.successful.clone();
    let complete = response.complete.clone();
    let data = response.data.clone();

    let do_it = move || {
        let client = Client::new();
        let mut http_response = match client.get(&url).header(Connection::close()).send() {
            Ok(n) => n,
            Err(e) => {
                let mut complete_unwrapped = complete.lock().unwrap();
                (*complete_unwrapped) = true;
                println!("(x)> Error getting {}: {}", url, e);
                return;
            }
        };

        let mut body_data = Vec::new();
        match http_response.read_to_end(&mut body_data) {
            Ok(_) => (),
            Err(e) => {
                let mut complete_unwrapped = complete.lock().unwrap();
                (*complete_unwrapped) = true;
                println!("(x)> Error getting {}: {}", url, e);
                return;
            }
        };

        body_data.push(0);

        let mut data_unwrapped = data.lock().unwrap();
        let mut successful_unwrapped = successful.lock().unwrap();
        let mut complete_unwrapped = complete.lock().unwrap();
        (*data_unwrapped) = Some(body_data);
        (*successful_unwrapped) = true;
        (*complete_unwrapped) = true;
    };

    if !async {
        do_it();
    }
    else {
        std::thread::spawn(move || { do_it() });
    }

    Box::into_raw(Box::new(response))
}

#[no_mangle]
pub unsafe extern fn http_wait_async(response : *const HttpResponse) {
    loop {
        let complete = (*response).is_complete();
        if complete {
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern fn http_destroy_response(response : *mut HttpResponse) {
    Box::from_raw(response);
}

#[no_mangle]
pub unsafe extern fn http_response_received(response : *mut HttpResponse) -> bool {
    (*response).is_complete()
}

#[no_mangle]
pub unsafe extern fn http_response_is_null(response : *const HttpResponse) -> bool {
    !(*response).is_complete() || !(*response).is_successful()
}

#[no_mangle]
pub unsafe extern fn http_read_response(response : *const HttpResponse) -> *const u8 {
    let data_arc = (*response).data.clone();
    let unlocked = data_arc.lock().unwrap();
    unlocked.as_ref().unwrap().as_ptr()
}

#[no_mangle]
pub unsafe extern fn http_response_length(response : *mut HttpResponse) -> u32 {
    let data_arc = (*response).data.clone();
    let unlocked = data_arc.lock().unwrap();

    if unlocked.is_none() {
        return 0;
    }
    unlocked.as_ref().unwrap().len() as u32 - 1
}
