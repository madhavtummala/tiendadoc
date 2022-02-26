mod database_utils;
mod telegram_utils;

use std::env;
use frankenstein::{Api, Message, SendMessageParams, TelegramApi};
use frankenstein::api_params::GetUpdatesParamsBuilder;
use rusqlite::{Connection, Result, Statement};

use telegram_utils::{reply_add, reply_start, reply_help, reply_auth, reply_search, reply_unknown};

pub fn get_env_variables() -> (String, String, String) {
    let bot_token = env::var("BOT_TOKEN").unwrap_or_default();
    let password = env::var("PASSWD").unwrap_or_default();
    let chats = env::var("CHAT_FILTER").unwrap_or_default();
    (bot_token, password, chats)
}

fn run_bot(api: &Api, conn: &Connection, vars: (String, String, String)) {
    let mut authenticated = false;
    let chats:Vec<String> = vars.2.split(",").map(|s| s.to_string()).collect();
    let mut update_params_builder = GetUpdatesParamsBuilder::default();
    update_params_builder.allowed_updates(vec!["message".to_string()]);
    let mut update_params = update_params_builder.build().unwrap();

    loop {
        match api.get_updates(&update_params) {
            Ok(response) => {
                for update in response.result {
                    if let Some(msg) = update.message {
                        // check if the message has a command
                        let command =  msg.text.as_ref();
                        let chat_filter = chats.is_empty() ||
                            chats.contains(&msg.chat.id.to_string());
                        if chat_filter && !command.is_none() && command.unwrap().starts_with("/") {
                            // calculate the reply, performing the commands
                            let reply = match command.unwrap().split(' ').next().unwrap() {
                                "/start" => match reply_start(msg, &vars.1) {
                                    (r, true) => {
                                        authenticated = true;
                                        r
                                    },
                                    (r, false) => {
                                        authenticated = false;
                                        r
                                    }
                                }
                                "/help" => reply_help(msg),
                                "/add" => match authenticated {
                                    true => {
                                        reply_add(msg, conn, api)
                                    },
                                    false => reply_auth(msg)
                                }
                                "/search" => match authenticated {
                                    true => reply_search(msg, conn, api),
                                    false => reply_auth(msg)
                                }
                                _ => reply_unknown(msg)
                            };
                            // send the reply
                            match reply {
                                Some(reply) => {
                                    if let Err(err) = api.send_message(&reply) {
                                        println!("Failed to send message: {:?}", err);
                                    }
                                },
                                None => ()
                            }
                        }
                    }
                    // check for only next messages
                    update_params = update_params_builder
                        .offset(update.update_id + 1)
                        .build()
                        .unwrap();
                }
            }
            Err(error) => {
                println!("Failed to get updates: {:?}", error);
            }
        }
    }
}

fn main() -> Result<()> {
    let conn = database_utils::get_connection("/usr/app/config/tiendadoc.db")?;
    let vars = get_env_variables();
    let api = Api::new(&vars.0);
    run_bot(&api, &conn, vars);
    Ok(())
}