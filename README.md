# tiendadoc
A telegram bot to tag files and search them later  
My first project in Rust ❤️

## Installation
Build from source
```bash
git clone https://github.com/madhavtummala/tiendadoc.git
cd tiendadoc
cargo build
./target/debug/tiendadoc
```

Docker Compose
```yaml
  tiendadoc:
    image: ghcr.io/madhavtummala/tiendadoc
    container_name: tiendadoc
    environment:
      - PASSWD= #optional
      - BOT_TOKEN= #telegram bot token
      - CHAT_FILTER= #comma separated chat ids
    volumes:
      - ${BASE_ROOT}/config/tiendadoc:/usr/app/config #tiendadoc.db (sqlite) needs to persist
    restart: unless-stopped
```

## Usage
You can add files in a chat with the bot or in a group which has the bot by replying to the media item (image, video, document, audio) with the `/add` command. The files are already uploaded to telegram servers and will remain for 1 year, they are not stored separately by this code. The file id (referenced by telegram) is used directly to reply for `/search` command. `/search` replies with all items matching atleast one tag.  

Commands:
```
- /start <password>
- /add <tag1> <tag2> ...
- /search <tag1> <tag2> ...
- /help
```

## Contributing
For anyone trying to learn rust, or trying to optimize / correct the code, please feel free to open a PR.  
