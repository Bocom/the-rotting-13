extern crate discord;

use discord::{Discord, ChannelRef, State};
use discord::model::Event;

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

    let (mut connection, ready) = discord.connect().expect("connect failed");
    let mut state = State::new(ready);

    println!("Connected!");
    
    loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(discord::Error::Closed(code, body)) => {
                println!("[Error] Connection closed with status {:?}: {}", code, body);
                break
            },
            Err(err) => {
                println!("[Warning] Receive error: {:?}", err);
                continue
            }
        };
        state.update(&event);

        match event {
            Event::MessageCreate(message) => {
                match state.find_channel(&message.channel_id) {
                    Some(ChannelRef::Private(channel)) => {
                        if message.author.name != channel.recipient.name {
                            continue
                        }

                        let original_message = message.content;
                        let mut new_message = String::new();
                        for chr in original_message.chars() {
                            new_message.push(rot13(chr));
                        }

                        let _ = discord.send_message(&message.channel_id, &new_message, "", false);
                    },
                    None => println!("Got a message from an unknown channel??? From {} saying {}", message.author.name, message.content),
                    _ => {},
                }
            },
            _ => {}
        }
    }
}