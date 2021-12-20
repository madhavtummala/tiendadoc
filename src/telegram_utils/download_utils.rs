use std::path::Path;
use std::fs::File;
use std::io::Write;
use frankenstein::{Api, GetFileParams, Message, TelegramApi};
use frankenstein::api_params::GetFileParamsBuilder;

pub fn download_telegram_file(api: &Api, bot_token:String, file_id: &String) -> Result<(), frankenstein::Error> {
    let get_file_params = GetFileParamsBuilder::default()
        .file_id(file_id).build().unwrap();
    let file_path = Some(api.get_file(&get_file_params)?.result.file_path).unwrap().unwrap();
    let url = format!("https://api.telegram.org/file/bot{}/{}", bot_token, file_path);
    if let Err(e) = download_file(url, file_id) {
        println!("Download file failed");
    }
    Ok(())
}

pub fn download_file(url: String, name: &String) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let path = format!("downloads/{}.pdf", name);
    let mut file = File::create(&Path::new(path.as_str()));
    if file.is_ok() {
        let content = response.text()?;
        file.unwrap().write_all(content.as_bytes())?;
    } else {
        println!("file couldn't be created");
    }
    Ok(())
}