extern crate wapc_guest as guest;

use guest::prelude::*;

#[no_mangle]
pub extern "C" fn wapc_init() {
  register_function("hello_world", hello_world);
}

fn hello_world(_msg: &[u8]) -> CallResult {
    let mut res = host_call("host", "redis", "GET", b"some_key1")?;
    res.extend(host_call("host", "redis", "GET", b"some_key2")?);
    res.extend(host_call("host", "redis", "GET", b"some_key3")?);
    Ok(res)
}
