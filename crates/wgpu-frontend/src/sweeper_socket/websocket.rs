use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use log::{error, info};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::__rt::IntoJsResult;
use web_sys::{js_sys, BinaryType, ErrorEvent, MessageEvent, WebSocket};
use world::client_messages::ClientMessage;
use world::{World};
use world::server_messages::{ServerMessage};
use crate::sweeper_socket::SweeperSocket;

pub struct WebSocketWorld {
    world: World,
    receiver: Receiver<ServerMessage>,
    web_socket: WebSocket,
    message_queue: VecDeque<ClientMessage>,
    connected: bool
}

impl WebSocketWorld {
    pub fn new() -> Result<WebSocketWorld, JsValue> {
        let ws = WebSocket::new("/ws")?;
        // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
        ws.set_binary_type(BinaryType::Arraybuffer);
        
        let (tx, rx) = mpsc::channel();
        
        let tx_clone = tx.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                info!("message event, received array buffer: {:?}", abuf);
                let array = js_sys::Uint8Array::new(&abuf);
                match ServerMessage::from_compressed(array.to_vec()) {
                    Ok(message) => {
                        let _ = tx_clone.send(message);
                    }
                    Err(_) => {
                        error!("Error receiving event")
                    }
                }
            } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                info!("message event, received blob: {:?}", blob);
                // I don't think I'm going to be using blob
            } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                info!("message event, received Text: {:?}", txt);
            } else {
                info!("message event, received Unknown: {:?}", e.data());
            }
        });
        // set message event handler on WebSocket
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // forget the callback to keep it alive
        onmessage_callback.forget();

        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            info!("error event: {:?}", e);
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let tx_clone = tx.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            info!("socket opened");
            tx_clone.send(ServerMessage::Connected).unwrap();
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        Ok(Self {
            world: World::new(),
            receiver: rx,
            web_socket: ws,
            message_queue: Default::default(),
            connected: false,
        })
    }
}

impl SweeperSocket for WebSocketWorld {
    fn send(&mut self, message: ClientMessage) {
        self.message_queue.push_back(message);
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        // Send any messages in the queue:
        if self.connected {
            while let Some(message) = self.message_queue.pop_front() {
                if let Ok(message) = (|| {
                    let js_message = serde_wasm_bindgen::to_value(&message).into_js_result()?;
                    js_sys::JSON::stringify(&js_message)
                })() {
                    info!("Sending message: {:?}", message);
                    match self.web_socket.send_with_str(&message.as_string().unwrap_or("".to_string())) {
                        Ok(_) => {}
                        Err(_) => error!("Message could not be sent")
                    }
                } else {
                    error!("Message could not be serialized");
                }
            }
        }
        
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                ServerMessage::Connected => {
                    self.connected = true;
                }
                _ => {}
            }
            Some(message)
        } else { None }
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}