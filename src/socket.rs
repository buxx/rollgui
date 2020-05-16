extern crate websocket;

use self::websocket::OwnedMessage;
use mpsc::SendError;
use serde_json::Value;
use std::io::Error;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::SystemTime;
use websocket::{ClientBuilder, Message, WebSocketError};

use crate::event;

pub struct ZoneSocket {
    ws_address: String,
    from_main_sender: mpsc::Sender<event::ZoneEvent>,
    from_main_receiver: Arc<Mutex<mpsc::Receiver<event::ZoneEvent>>>,
    from_websocket_sender: Arc<Mutex<mpsc::Sender<event::ZoneEvent>>>,
    from_websocket_receiver: mpsc::Receiver<event::ZoneEvent>,
    ws_reader_handle: Option<JoinHandle<()>>,
    ws_reader_closed: Arc<Mutex<bool>>,
    ws_sender_handle: Option<JoinHandle<()>>,
    ws_sender_closed: Arc<Mutex<bool>>,
    closing: bool,
}

impl ZoneSocket {
    pub fn new(ws_address: String) -> Self {
        let (from_main_sender, from_main_receiver) = mpsc::channel();
        let (from_websocket_sender, from_websocket_receiver) = mpsc::channel();
        let from_main_receiver = Arc::new(Mutex::new(from_main_receiver));
        let from_websocket_sender = Arc::new(Mutex::new(from_websocket_sender));
        let ws_reader_closed = Arc::new(Mutex::new(false));
        let ws_sender_closed = Arc::new(Mutex::new(false));

        Self {
            ws_address,
            from_main_sender,
            from_main_receiver,
            ws_reader_handle: None,
            ws_reader_closed,
            ws_sender_handle: None,
            ws_sender_closed,
            from_websocket_sender,
            from_websocket_receiver,
            closing: false,
        }
    }

    pub fn connect(&mut self) {
        let from_main_receiver = Arc::clone(&self.from_main_receiver);
        let from_websocket_sender = Arc::clone(&self.from_websocket_sender);
        let ws_reader_closed = Arc::clone(&self.ws_reader_closed);
        let ws_sender_closed = Arc::clone(&self.ws_sender_closed);

        let ws_client = ClientBuilder::new(self.ws_address.as_str())
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (mut ws_reader, mut ws_writer) = ws_client.split().unwrap();

        // ws reader
        let ws_reader_handle = thread::spawn(move || {
            let from_websocket_sender = from_websocket_sender.lock().unwrap();
            for message in ws_reader.incoming_messages() {
                match message {
                    Ok(OwnedMessage::Text(msg)) => {
                        let value: Value = serde_json::from_str(&msg).unwrap();
                        let event = event::ZoneEvent::from_value(value).unwrap();

                        let mut break_ = false;
                        if let event::ZoneEventType::ServerPermitClose = event.event_type {
                            break_ = true;
                        }

                        if let Err(SendError(_e)) = from_websocket_sender.send(event) {
                            eprintln!("WebSocket(receiver): Something went wrong during process of received event");
                        }

                        if break_ {
                            println!("WebSocket(receiver): Receive close event");
                            break;
                        }
                    }
                    Ok(OwnedMessage::Close(_)) => {
                        println!("WebSocket(receiver): Close");
                        break;
                    }
                    Err(WebSocketError::NoDataAvailable) => {
                        println!("WebSocket(receiver): WebSocketError: NoDataAvailable");
                        break;
                    }
                    _ => eprintln!(
                        "WebSocket(receiver): Unknown websocket message received: {:?}",
                        message
                    ), // TODO add ping/pong (OwnedMessage::ping|pong)
                }
            }

            let mut closed = ws_reader_closed.lock().unwrap();
            *closed = true;
            println!("WebSocket(receiver): Closing ...");
        });

        // ws sender
        let ws_sender_handle = thread::spawn(move || {
            let from_main_receiver = from_main_receiver.lock().unwrap();

            for received in from_main_receiver.iter() {
                let message_json_str = serde_json::to_string(&received).unwrap();
                let message = Message::text(message_json_str);
                ws_writer.send_message(&message).unwrap();

                if let event::ZoneEventType::ClientWantClose = received.event_type {
                    // Get out for loop (and finish thread)
                    println!("WebSocket(sender): Closing ...");
                    break;
                }
            }

            let mut closed = ws_sender_closed.lock().unwrap();
            *closed = true;
        });

        self.ws_reader_handle = Some(ws_reader_handle);
        self.ws_sender_handle = Some(ws_sender_handle);
    }

    pub fn send(&self, event: event::ZoneEvent) {
        self.from_main_sender.send(event).unwrap();
    }

    pub fn pending_events(&self) -> Vec<event::ZoneEvent> {
        let mut events: Vec<event::ZoneEvent> = Vec::new();
        let iter = &mut self.from_websocket_receiver.try_iter();
        while let Some(val) = iter.next() {
            events.push(val);
        }
        events
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.closing = true;
        self.send(event::ZoneEvent {
            event_type: event::ZoneEventType::ClientWantClose,
            event_type_name: String::from(event::CLIENT_WANT_CLOSE),
        });

        // FIXME: make a loop to look at booleans and exit when ok or when timeout
        let start = SystemTime::now();
        let timeout = Duration::from_secs(5);
        loop {
            let ws_sender_closed = *self.ws_sender_closed.lock().unwrap();
            let ws_reader_closed = *self.ws_reader_closed.lock().unwrap();

            if ws_sender_closed && ws_reader_closed {
                break;
            }

            if start.elapsed().unwrap() > timeout {
                eprintln!(
                    "WebSockets: timeout reached, force closing (ws_sender_closed: {}, ws_reader_closed: {})",
                    ws_sender_closed, ws_reader_closed
                );
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }
}
