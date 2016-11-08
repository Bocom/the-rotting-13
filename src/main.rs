extern crate discord;
extern crate websocket;

use discord::{Discord, ChannelRef, State};
use discord::model::Event;
use websocket::result::WebSocketError;

fn rot13(c: char) -> char {
    let base: u8 = match c {
        'a'...'z' => 'a' as u8,
        'A'...'Z' => 'A' as u8,
        _ => return c
    };

    let ordinal = c as u8 - base;
    let rot = (ordinal + 13) % 26;
    (rot + base) as char
}

fn main() {
    use std::env;
    println!("The Rotting 13!");

    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN").expect("Expected token")
    ).expect("login failed!");

    let (mut connection, ready) = discord.connect().expect("Connection failed");
    let mut state = State::new(ready);

    println!("Connected!");
    
    loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(err) => {
                if let discord::Error::WebSocket(ws_err) = err {
                    match ws_err {
                        WebSocketError::IoError(io) => {},
                        _ => {
                            // We were disconnected, try to reconnect
                            println!("Reconnecting...");
                            let (new_connection, ready) = discord.connect().expect("Reconnect failed");
                            connection = new_connection;
                            state = State::new(ready);
                            println!("Reconnected!");
                        },
                    }
                    continue
                }

                if let discord::Error::Closed(code, body) = err {
                    println!("[Error] Connection closed with status {:?}: {}", code, body);
                    break
                }

                println!("[Warning] Receive error: {:?}", err);
                continue
            }
        };
        state.update(&event);

        match event {
            Event::MessageCreate(message) => {
                if message.author.id != state.user().id {
                    continue
                }

                match state.find_channel(&message.channel_id) {
                    Some(ChannelRef::Private(channel)) => {
                        let original_message = message.content;
                        let mut new_message = String::new();
                        for chr in original_message.chars() {
                            new_message.push(rot13(chr));
                        }

                        let _ = discord.send_message(&channel.id, &new_message, "", false);
                    },
                    Some(ChannelRef::Public(server, channel)) => {

                    },
                    None => println!("Got a message from an unknown channel??? From {} saying {}", message.author.name, message.content),
                    _ => {},
                }
            },
            _ => {}
        }
    }
}