use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use js_sys::Uint8Array;
use log::{log, Record};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use crate::socket::MessageType::{Bytes, Opened, Text};
use crate::socket::SocketState::Open;

pub enum MessageType {
    Bytes(Vec<u8>),
    // Blob(),
    Text(String),
    Opened,
}

pub enum SocketState {
    Connecting,
    Open,
    Closed,
}

pub struct Socket {
    tx: Sender<MessageType>,
    pub rx: Receiver<MessageType>,
    ws: WebSocket,
    state: SocketState,
}

impl Socket {
    pub fn new() -> Result<Self, JsValue> {
        let (tx, rx) = mpsc::channel();
        // Connect to an echo server
        let ws = WebSocket::new("/")?;
        // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let new_socket = Self {
            tx: tx.clone(),
            rx,
            ws: ws.clone(),
            state: SocketState::Connecting,
        };

        // create callback
        let onmessage_tx = tx.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            // Handle difference Text/Binary,...
            if let Ok(array_buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = Uint8Array::new(&array_buf);
                onmessage_tx.send(Bytes(array.to_vec())).expect("Couldn't put bytes in channel");
            } else if let Ok(_blob) = e.data().dyn_into::<web_sys::Blob>() {
                // TODO: handle blob if we need to
            } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                if let Some(text) = txt.as_string() {
                    onmessage_tx.send(Text(text)).expect("Couldn't put text in channel");
                }
            } else {
                log::error!("message event, received Unknown: {:?}", e.data());
            }
        });
        // set message event handler on WebSocket
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // forget the callback to keep it alive
        onmessage_callback.forget();

        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            log::error!("error event: {:?}", e);
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onopen_tx = tx.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            onopen_tx.send(Opened).expect("Couldn't send opened message");
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        Ok(new_socket)
    }

    pub fn send(&mut self, message_type: MessageType) -> Result<(), JsValue> {
        return match message_type {
            Bytes(bytes) => {
                self.ws.send_with_u8_array(bytes.as_slice())
            }
            Text(text) => {
                self.ws.send_with_str(text.as_str())
            }
            _ => {
                Ok(())
            }
        }
    }

    pub fn get_messages(&mut self) -> Vec<MessageType> {
        let mut messages = vec![];
        loop {
            if let Ok(message) = self.rx.try_recv() {
                match message {
                    Opened => {
                        self.state = Open;
                    }
                    _ => {
                        messages.push(message);
                    }
                }
            } else {
                return messages;
            }
        }
    }
}