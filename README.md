# Touch the world
## What's this
A proxy tool implemented by rust.
## Build
You need a rust build environment.
```shell
cargo build --release
```
## How to use
 1. Creat a new config file `config.json`.
    Such as:
```json
{
  "input": {
    "name": "socks5",
    "config": {
      "local_host": "127.0.0.1",
      "local_port": 1080
    }
  },
  "output": {
    "name": "ss-aes-256-gcm",
    "config": {
      "remote_host": "127.0.0.1",
      "remote_port": 3391
    }
  }
}
```
 2. Make sure you use this directory structure.
```
(root dir)
    ├--config
    |   └--config.json
    └--touch-rust.exe / touch-rust
```
 3. run `./touch-rust`

## Status
|        protocol         |support|
|           :---:         | :---: |
|          SOCKS5         |   ✅  |
|    Shadowsocks AEAD     |   ✅  |
|   HTTP proxy support    |   ❌  |
|       UDP support       |   ❌  |
| More protocol support...|Coming soon...|

