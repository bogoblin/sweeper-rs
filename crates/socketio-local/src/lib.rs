use wasm_bindgen::prelude::*;
use web_sys::js_sys::Object;

#[wasm_bindgen(module = "socket.io.esm.min.js")]
extern "C" {
    pub type SocketIo;
    pub type Io;
    pub type Engine;
    
    pub fn io(url: &str) -> SocketIo;
    
    #[wasm_bindgen(method)]
    pub fn on(this: &SocketIo, event: &str, f: &dyn Fn(JsValue));
    
    #[wasm_bindgen(method, getter)]
    pub fn io(this: &SocketIo) -> Io;
    
    #[wasm_bindgen(method)]
    pub fn emit(this: &SocketIo, event: &str, data: JsValue);
    
    #[wasm_bindgen(method, getter)]
    pub fn engine(this: &Io) -> Engine;
    
    #[wasm_bindgen(method)]
    pub fn on(this: &Engine, event: &str, closure: &Closure<dyn FnMut(Object)>);
}