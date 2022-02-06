mod download_utils;

use std::hash::Hash;
use std::collections::{HashSet, HashMap};
use std::ptr::null;
use frankenstein::{Api, GetFileParams, Message, SendMessageParams, TelegramApi};
use frankenstein::api_params::{SendMessageParamsBuilder, GetFileParamsBuilder, InputFile};
use rusqlite::{Connection, Result, params, Statement};
use frankenstein::{SendDocumentParamsBuilder, SendPhotoParamsBuilder, SendVideoParamsBuilder, SendAudioParamsBuilder};

pub fn get_file_id(message: Option<&Box<Message>>) -> Option<String> {
    if message.is_some() {
        let document = message.unwrap().document.as_ref();
        let photo = message.unwrap().photo.as_ref();
        let video = message.unwrap().video.as_ref();
        let audio = message.unwrap().audio.as_ref();
        if document.is_some() {
            return Some(document.unwrap().file_id.to_string());
        } else if photo.is_some() {
            return Some(photo.unwrap().last().unwrap().file_id.to_string());
        } else if video.is_some() {
            return Some(video.unwrap().file_id.to_string());
        } else if audio.is_some() {
            return Some(audio.unwrap().file_id.to_string());
        }
    }
    Option::None
}

pub fn reply_unknown(message: Message) -> Option<SendMessageParams> {
    prepare_reply(message, "unknown command, please check /help")
}

pub fn reply_auth(message: Message) -> Option<SendMessageParams> {
    prepare_reply(message, "please authenticate with /start")
}

pub fn prepare_reply(message: Message, reply: &str) -> Option<SendMessageParams> {
    Some(SendMessageParamsBuilder::default()
        .chat_id(message.chat.id)
        .text(reply)
        .reply_to_message_id(message.message_id)
        .build()
        .unwrap())
}

pub fn reply_help(message: Message) -> Option<SendMessageParams> {
    let reply = "Store(es=Tienda) and search your (doc)s\n\
        /start <password> to authenticate\n\
        /add <key1 key2 ..> (reply) to a doc\n\
        /search <key1 key2 ..> among existing docs\n";
    prepare_reply(message, reply)
}

pub fn reply_add(message: Message, conn: &Connection, _api: &Api) -> Option<SendMessageParams> {
    let mut insert_keyword
        = conn.prepare("insert into keys (keyword, file_id) values (?1, ?2)").ok()?;
    let mut search_keyword
        = conn.prepare("select keyword, file_id from keys where keyword is (?1)").ok()?;
    let reply;
    match get_file_id(message.reply_to_message.as_ref()) {
        Some(file_id) => {
            // downloading is optional
            // if let Err(e) =
            // download_utils::download_telegram_file(_api, bot_token.to_string(), &file_id) {
            //     println!("extracting file path failed");
            // }
            let text = message.text.as_ref().unwrap();
            let keywords:Vec<String> = text.split(" ").skip(1)
                .map(|s| s.to_string().to_uppercase()).collect();
            for keyword in keywords {
                let result =
                    search_keyword.query_map(params![keyword],
                                             |row| {row.get(0)});
                if result.is_err() {
                    // first time inserting this keyword
                    if let Err(e) = insert_keyword.insert(params![keyword, file_id]) {
                        println!("insert keyword failed {:?}", e);
                    }
                } else {
                    let file_ids:Vec<String> = result.unwrap()
                        .map(|f| f.unwrap()).collect();
                    if !file_ids.contains(&file_id) {
                        if let Err(e) =
                        insert_keyword.execute(params![keyword, file_id]) {
                            println!("insert keyword failed {:?}", e);
                        }
                    }
                }
            }
            reply = "added the document to database";
        }
        None => {reply = "reply to an existing file in chat";}
    }
    prepare_reply(message, reply)
}

pub fn reply_search(message: Message, conn: &Connection, api: &Api) -> Option<SendMessageParams> {
    let mut search_keyword
        = conn.prepare("select file_id from keys where keyword is (?1)").ok()?;
    let text = message.text.as_ref().unwrap();
    let keywords:Vec<String> = text.split(" ").skip(1)
        .map(|s| s.to_string().to_uppercase()).collect();
    let mut filtered_files: Option<HashSet<String>> = None;
    let reply;
    for keyword in keywords {
        let result =
            search_keyword.query_map(params![keyword],
                                     |row| {row.get(0)});
        if result.is_ok() {
            let file_ids:HashSet<String> = result.unwrap()
                .map(|f| f.unwrap()).collect();
            if filtered_files.is_some() {
                filtered_files = Some(intersection(filtered_files.unwrap(), &file_ids));
            } else {
                filtered_files = Some(file_ids.into_iter().collect());
            }
        }
    }
    match filtered_files {
        Some(ref set) => {
            reply = "all matching files sent";
            for file in set {
                let send_document_params = SendDocumentParamsBuilder::default()
                    .chat_id(message.chat.id)
                    .reply_to_message_id(message.message_id)
                    .document(frankenstein::api_params::File::String(file.to_string()))
                    .build().unwrap();
                if let Err(_e) = api.send_document(&send_document_params) {
                    let send_photo_params = SendPhotoParamsBuilder::default()
                        .chat_id(message.chat.id)
                        .reply_to_message_id(message.message_id)
                        .photo(frankenstein::api_params::File::String(file.to_string()))
                        .build().unwrap();
                    if let Err(_e) = api.send_photo(&send_photo_params) {
                        let send_video_params = SendVideoParamsBuilder::default()
                            .chat_id(message.chat.id)
                            .reply_to_message_id(message.message_id)
                            .video(frankenstein::api_params::File::String(file.to_string()))
                            .build().unwrap();
                        if let Err(_e) = api.send_video(&send_video_params) {
                            let send_audio_params = SendAudioParamsBuilder::default()
                                .chat_id(message.chat.id)
                                .reply_to_message_id(message.message_id)
                                .audio(frankenstein::api_params::File::String(file.to_string()))
                                .build().unwrap();
                            api.send_audio(&send_audio_params).ok()?;
                        }
                    }
                }
            }
        }
        None => reply = "no matching files found"
    };
    prepare_reply(message, reply)
}

pub fn reply_start(message: Message, password: &String) -> (Option<SendMessageParams>, bool) {
    let text = message.text.as_ref().unwrap();
    let items:Vec<String>= text.split(' ').map(|s| s.to_string()).collect();
    let reply;
    let status;
    if items.get(1).is_some() {
        if password == items.get(1).unwrap() {
            reply = "successfully authenticated";
            status = true;
        } else {
            reply = "wrong password";
            status = false;
        }
    } else if !password.is_empty() {
        reply = "please enter a password with /start";
        status = false;
    } else {
        reply = "successfully authenticated";
        status = true;
    }
    (prepare_reply(message, reply), status)
}

pub fn intersection<T: Eq + Hash>(a: HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.into_iter().filter(|e| b.contains(e)).collect()
}