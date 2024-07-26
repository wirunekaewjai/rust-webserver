mod enums;
mod functions;
mod structs;

use std::{collections::{HashMap, VecDeque}, io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, ops::Sub, sync::{Arc, Mutex, RwLock}, thread, time::Duration};

use enums::{ConnectionState, ProtocolVersion, RequestMethod};
use serde::Deserialize;
use serde_json::{json, Deserializer};
use structs::{Connection, Context, Response};

use crate::shared;

pub fn start_server() {
    let hostname = [0, 0, 0, 0];
    let port = 8080;

    let addr = SocketAddr::from((hostname, port));
    let listener = TcpListener::bind(addr).expect("> failed to bind tcp listener");

    listener.set_nonblocking(true).expect("> failed to set non-blocking");

    println!(
        "\n> Running at {}:{}\n",
        hostname.map(|v| v.to_string()).join("."),
        port
    );

    let thread_count = match thread::available_parallelism() {
        Ok(value) => value.get().sub(1).max(1),
        Err(_) => 1,
    };

    let connections = Arc::new(Mutex::new(VecDeque::<Connection>::new()));
    let subscriptions = Arc::new(RwLock::new(HashMap::<String, HashMap<u128, TcpStream>>::new()));

    for _ in 0..thread_count {
        let connections = connections.clone();
        let context = Context {
            subscriptions: subscriptions.clone(),
        };

        let _ = thread::spawn(move || {
            let sleep_duration = Duration::from_micros(10);
            let mut buffer = [0; 4096];

            loop {
                let mut connections_guard = connections.lock().unwrap();
                let Some(mut connection) = connections_guard.pop_front() else {
                    drop(connections_guard);

                    thread::sleep(sleep_duration);
                    continue;
                };

                drop(connections_guard);

                match connection.stream.read(&mut buffer) {
                    Ok(size) => {
                        if size > 0 {
                            connection.buffer.extend_from_slice(&buffer[..size]);

                            if connection.websocket {
                                handle_ws(&context, &mut connection);
                            } else {
                                handle_http(&context, &mut connection);
                            }

                            {
                                connections.lock().unwrap().push_back(connection);
                            }

                            continue;
                        }

                        if let Ok(addr) = connection.stream.peer_addr() {
                            println!("> {}: [TCP] closed", addr);
                        };

                        // connection closed
                        continue;
                    }

                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            {
                                connections.lock().unwrap().push_back(connection);
                            }

                            // running in non-blocking mode will got this err.
                            continue;
                        }

                        eprintln!("> {}: error: {}", addr, e);
                        continue;
                    }
                }
            }
        });
    }

    let mut connection_id: u128 = 0;

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("> {}: [TCP] connected", addr);

                let Ok(_) = stream.set_nonblocking(true) else {
                    eprintln!("> failed to set non-blocking mode to stream");
                    continue;
                };

                connection_id += 1;

                let mut connections_guard = connections.lock().unwrap();

                connections_guard.push_back(Connection {
                    id: connection_id,

                    stream,
                    buffer: Vec::new(),

                    websocket: false,

                    http_content_length: 0,
                    http_headers: HashMap::new(),
                    http_headers_length: 0,
                    http_method: RequestMethod::None,
                    http_request_uri: String::new(),
                    http_state: ConnectionState::ReadProtocol,
                    http_version: ProtocolVersion::None,
                });

                drop(connections_guard);
            }

            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    // running in non-blocking mode will got this err.
                    continue;
                }

                eprintln!("> accept error: {}", e);
                return;
            }
        }
    }
}

fn handle_http(
    context: &Context,
    connection: &mut Connection,
) {
    if connection.http_state == ConnectionState::ReadProtocol {
        if let Err(response) = functions::read_protocol(connection) {
            return functions::write_response(connection, &response);
        }
    }

    if connection.http_state == ConnectionState::ReadHeaders {
        if let Err(response) = functions::read_headers(connection) {
            return functions::write_response(connection, &response);
        }
    }

    if connection.http_state == ConnectionState::ValidateHeaders {
        let method = &connection.http_method;
        let request_uri = &connection.http_request_uri;

        let path_and_qs = request_uri.split("?").collect::<Vec<&str>>();
        let path = path_and_qs[0];

        if method == &RequestMethod::Get && path == "/" {
            if connection.http_content_length > 0 {
                return functions::write_response(connection, &Response::not_acceptable());
            } else {
                connection.http_state = ConnectionState::HandleRequest;
            }
        }

        else if method == &RequestMethod::Post && path == "/create" {
            if connection.http_content_length > 0 {
                connection.http_state = ConnectionState::ReadBody;
            } else {
                return functions::write_response(connection, &Response::not_acceptable());
            }
        }
        
        else {
            return functions::write_response(connection, &Response::not_found());
        }
    }

    if connection.http_state == ConnectionState::ReadBody {
        if let Err(response) = functions::read_body(connection) {
            return functions::write_response(connection, &response);
        }
    }

    if connection.http_state == ConnectionState::HandleRequest {
        let method = &connection.http_method;
        let request_uri = &connection.http_request_uri;

        let path_and_qs = request_uri.split("?").collect::<Vec<&str>>();
        let path = path_and_qs[0];

        if method == &RequestMethod::Get && path == "/" && connection.websocket {
            return functions::write_response(connection, &functions::ws::handle_upgrade(connection));
        }   

        else if method == &RequestMethod::Post && path == "/create" {
            let mut deserializer = Deserializer::from_reader(&*connection.buffer);
            let Ok(input) = shared::structs::CreateRoomInput::deserialize(&mut deserializer) else {
                return functions::write_response(connection, &Response::bad_request());
            };

            let mut subscriptions_guard = context.subscriptions.write().unwrap();

            if !subscriptions_guard.contains_key(&input.name) {
                subscriptions_guard.insert(input.name, HashMap::new());
            }

            drop(subscriptions_guard);

            return functions::write_response(connection, &Response::created());
        }
        
        else {
            return functions::write_response(connection, &Response::not_found());
        }
    }
}

fn handle_ws(
    context: &Context,
    connection: &mut Connection,
) {
    // read data frame from client.
    let Some((opcode, payload)) = functions::ws::read_frame(&mut connection.buffer) else {
        return;
    };

    if opcode == 8 {
        if let Ok(addr) = connection.stream.peer_addr() {
            println!("> {}: [WS] disconnected", addr);
        }

        let ok = functions::ws::build_frame(true, 8, b"");

        let _ = connection.stream.write_all(&ok);
        let _ = connection.stream.flush();

        connection.websocket = false;

        let mut subscriptions_guard = context.subscriptions.write().unwrap();

        for (channel, subscriptions) in subscriptions_guard.iter_mut() {
            subscriptions.remove(&connection.id);

            if let Ok(addr) = connection.stream.peer_addr() {
                println!("> {}: [WS] unsubscribe {}", addr, channel);
            }
        }

        drop(subscriptions_guard);

        return;
    }

    if opcode == 1 || opcode == 2 {
        if let Some(payload) = payload {
            let mut deserializer = serde_json::Deserializer::from_reader(&*payload);

            if let Ok(msg) = shared::enums::WsMessage::deserialize(&mut deserializer) {
                match msg {
                    shared::enums::WsMessage::SUBSCRIBE { room } => {
                        let mut subscriptions_guard = context.subscriptions.write().unwrap();

                        if !subscriptions_guard.contains_key(&room) {
                            let msg = shared::enums::WsMessage::SUBSCRIBE_REJECTED {
                                room,
                            };

                            let msg_json = json!(msg).to_string();
                            let msg_buffer = functions::ws::build_frame(true, 1, msg_json.as_bytes());

                            let _ = connection.stream.write_all(&msg_buffer);
                            let _ = connection.stream.flush();

                            return;
                        }

                        let subscriptions = subscriptions_guard.get_mut(&room).unwrap();

                        subscriptions.insert(connection.id, connection.stream.try_clone().unwrap());

                        drop(subscriptions_guard);

                        if let Ok(addr) = connection.stream.peer_addr() {
                            println!("> {}: [WS] subscribe to {}", addr, room);
                        }
                    },

                    shared::enums::WsMessage::UNSUBSCRIBE { room } => {
                        let mut subscriptions_guard = context.subscriptions.write().unwrap();
            
                        if subscriptions_guard.contains_key(&room) {
                            let subscriptions = subscriptions_guard.get_mut(&room).unwrap();

                            subscriptions.remove(&connection.id);
                        }

                        drop(subscriptions_guard);

                        if let Ok(addr) = connection.stream.peer_addr() {
                            println!("> {}: [WS] unsubscribe from {}", addr, room);
                        }
                    },

                    shared::enums::WsMessage::SEND { room, text } => {
                        let mut subscriptions = context.subscriptions.write().unwrap();
                        let msg = shared::enums::WsMessage::NOTIFY {
                            room: room.clone(),
                            text,
                        };
                    
                        let msg_json = json!(msg).to_string();
                        let msg_buffer = functions::ws::build_frame(true, 1, msg_json.as_bytes());
                    
                        if let Some(subscriptions) = subscriptions.get_mut(&room) {
                            for (id, stream) in subscriptions.iter_mut() {
                                if id == &connection.id {
                                    continue;
                                }

                                let _ = stream.write_all(&msg_buffer);
                                let _ = stream.flush();
                            }
                        }
                    
                        drop(subscriptions);
                    },

                    _ => {
                        if let Ok(addr) = connection.stream.peer_addr() {
                            println!("> {}: [WS] unknown message", addr);
                        }
                    },
                };
            }
        }
    }
}