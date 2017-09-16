extern crate discord;
extern crate websocket;

use discord::{Discord, ChannelRef, State};
use discord::model::{Event, User, ReactionEmoji, PossibleServer, ServerId};
use websocket::result::WebSocketError;

const REACTION_EMOJI: &'static str = "rot13";
const PLAYING_GAME_NAME: &'static str = "with words";

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

fn create_rot13_message(message: String) -> String {
    let mut new_message = String::new();
    for chr in message.chars() {
        new_message.push(rot13(chr));
    }

    new_message
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

    connection.set_game_name(String::from(PLAYING_GAME_NAME));
    
    loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(err) => {
                if let discord::Error::WebSocket(ws_err) = err {
                    match ws_err {
                        WebSocketError::IoError(_) => {},
                        _ => {
                            // We were disconnected, try to reconnect
                            println!("Reconnecting...");

                            let (new_connection, ready) = discord.connect().expect("Reconnect failed");
                            connection = new_connection;

                            state = State::new(ready);

                            println!("Reconnected!");
                            connection.set_game_name(String::from(PLAYING_GAME_NAME));
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
                if message.author.id == state.user().id {
                    continue
                }

                match state.find_channel(message.channel_id) {
                    Some(ChannelRef::Private(channel)) => {
                        let new_message = create_rot13_message(message.content);

                        let _ = discord.send_message(channel.id, &new_message, "", false);
                    },
                    Some(ChannelRef::Public(_, channel)) => {
                        // state.user() is not of the same type as message.mentions
                        // Is there a better way to do this?
                        let res: Vec<&User> = message.mentions.iter()
                            .filter(|&u| u.id == state.user().id)
                            .collect();

                        if res.len() > 0 {
                            let text_message = format!("I encode and decode ROT13 messages, just send me a DM with a message to encode or decode!\nYou can also react to a message with :{}: to get it sent as DM!", REACTION_EMOJI);
                            let _ = discord.send_message(channel.id, &text_message, "", false);
                        }
                    },
                    None => println!("Got a message from an unknown channel??? From {} saying {}", message.author.name, message.content),
                    _ => {},
                }
            },
            Event::ReactionAdd(reaction) => {
                let received_emoji = match reaction.emoji {
                    ReactionEmoji::Custom { name, .. } => name,
                    ReactionEmoji::Unicode(emoji) => emoji
                };

                if received_emoji != REACTION_EMOJI {
                    continue
                }

                let message = match discord.get_message(reaction.channel_id, reaction.message_id) {
                    Ok(msg) => msg,
                    Err(_) => {
                        println!("Could not find the message that was reacted to. Message ID {}", reaction.message_id);
                        continue
                    }
                };

                match discord.create_private_channel(reaction.user_id) {
                    Ok(channel) => {
                        let new_message = create_rot13_message(message.content);
                        
                        let _ = discord.send_message(channel.id, &new_message, "", false);
                    },
                    Err(_) => println!("Got an invalid reaction??? From user ID {} on message ID {}", reaction.user_id, reaction.message_id),
                }
            },
            Event::ServerCreate(possible_server) => {
                match possible_server {
                    PossibleServer::Online(server) => {
                        println!("Present in server {}", server.name);
                    },
                    PossibleServer::Offline(server_id) => {
                        let ServerId(id) = server_id;
                        println!("Present in offline server #{}", id);
                    }
                }
            },
            _ => {}
        }
    }
}