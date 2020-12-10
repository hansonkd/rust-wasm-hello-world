extern crate wapc_guest as guest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use serde::{Deserialize, Serialize};
use guest::prelude::*;
use serde::de::DeserializeOwned;
use std::sync::RwLock;
use std::sync::Arc;
use std::fmt;
use std::panic;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Human {
    age: u32,
    name: String,
}


#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Tookie {
    age: u32,
    name: String,
}


macro_rules! register_view {
    ($n:ident, $t:ident) => {{
        register_function(&["view-start-", stringify!($n)].join("")[..], |b: &[u8]| {
            let s = $t::start(b)?;
            serialize(&s)
        });
        register_function(&["view-event-", stringify!($n)].join("")[..], |b: &[u8]| {
            let (state, name, payload): (&[u8], &str, &[u8]) = deserialize(b)?;
            let mut s: $t = deserialize(state)?;
            s.event(name, payload)?;
            serialize(&s)
        });
        register_function(&["view-render-", stringify!($n)].join("")[..], |b: &[u8]| {
            let s: $t = deserialize(b)?;
            let html = s.render()?;
            Ok(html.into_bytes())

        });
    }};
}
macro_rules! register_call {
    ($n:ident, $f:ident) => {{
        register_function(&["call-", stringify!($n)].join("")[..], |b: &[u8]| {
            run_function(b, $f)
        });
    }};
}


#[no_mangle]
pub extern "C" fn wapc_init() {
    // register_view_generator(assemble::ROOT, root_view);
    // register_function("hello_world", handle_start, handle_event, handle_render);
    panic::set_hook(Box::new(hook));
    register_view!(first_view, ViewHandler);
    register_call!(first_call, some_call);
}

pub fn hook(info: &panic::PanicInfo) {
    let mut msg = info.to_string();
    console_log(&msg[..]);
}


#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Copy)]
struct MyState {
    age: u32,
}


pub type Html = String;
pub type AssembleResult<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

pub trait Summary: Sync + Send {
    fn start(params: &[u8]) -> AssembleResult<Self> where Self: Sized;
    fn event(&self, msg: &str, body: &[u8]) -> AssembleResult<()>;
    fn render(&self) -> AssembleResult<Html>;
}

#[derive(Deserialize, Serialize)]
struct ViewHandler {
    state: MyState
}

impl Summary for ViewHandler {

    fn start(params: &[u8]) -> AssembleResult<Self> {
        Ok(ViewHandler{state: MyState{age: 0}})
    }

    fn render(&self) -> AssembleResult<Html> {
        Ok(r#"
        <div>
            <h1>Hello World</h2>
            <button assemble-click="i-was-clicked">Click me</button>
        </div>
        "#.to_string())
    }

    fn event(&self, msg: &str, payload: &[u8]) -> AssembleResult<()> {
        Ok(())
    }
}


pub fn serialize<T: ?Sized>(val: &T) -> AssembleResult<Vec<u8>> where
    T: Serialize {
        match rmp_serde::to_vec(val) {
            Ok(v) => Ok(v),
            Err(v) => Err(Box::new(v)),
        }
}

pub fn deserialize<'a, R: ?Sized, T>(rd: &'a R) -> AssembleResult<T> where
    R: AsRef<[u8]>,
    T: Deserialize<'a> {
        match rmp_serde::from_read_ref(rd) {
            Ok(v) => Ok(v),
            Err(v) => Err(Box::new(v)),
        }
}



fn some_call(t: &Option<Human>) -> AssembleResult<Box<Tookie>> {
    let v = vec![0; 500 * 1024 * 1024];
    let mut res = host_call("host", "redis", "GET", b"some_key1")?;
    res.extend(host_call("host", "redis", "GET", b"some_key2")?);
    res.extend(host_call("host", "redis", "GET", b"some_key3")?);

    let took = Tookie {
        age: 42,
        name: std::str::from_utf8(&res).unwrap().into(),
    };
    Ok(Box::new(took))
}

pub fn run_function<T, R>(b: &[u8], f: fn(&Option<T>) -> std::result::Result<Box<R>, Box<dyn std::error::Error + Sync + Send>>) -> CallResult where T: DeserializeOwned, R: Serialize + ?Sized {
    if b.len() == 0 {
        match rmp_serde::to_vec(&f(&None)?) {
            Ok(v) => Ok(v),
            Err(v) => Err(Box::new(v)),
        }
    } else {
        match rmp_serde::from_read_ref(&b){
            Ok(input) => {
                match rmp_serde::to_vec(&f(&Some(input))?) {
                    Ok(v) => Ok(v),
                    Err(v) => Err(Box::new(v)),
                }
            }
            Err(v) => Err(Box::new(v)),
        }
    }

}
