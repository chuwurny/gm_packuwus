# ⚡ PackUwUs - lua pack system

Open-source alternative to [billy's gluapack](https://www.gmodstore.com/market/view/gluapack) and better version of [danielga's luapack](https://github.com/danielga/luapack)

## Features
- [x] Asynchronous lua packing
- [x] Simple Lua API
- [x] Lua auto refresh support
- [x] Safely disconnect client if any fatal error occurred
- [x] Custom `init.lua` support
- [x] Strip indents & trailing whitespaces
- [ ] Strip unnecessary whitespaces
- [ ] Remove comments

## Usage

**Firstly**, you need to setup FastDL (HTTP) server. [There's complete guide how to do it](https://github.com/synchronocy/How-to-Setup-Fastdl?tab=readme-ov-file#configuring-fastdl), but in short:
1. Install web-server (eg. nginx, apache)
2. Configure web-server to work as static content provider
3. Provide `garrysmod/data/serve_packuwus` directory
4. Set `sv_loadingurl <url>` convar in your `server.cfg`

**Secondly**, configure this addon.
1. Clone this repo into `addons` directory.
2. Rename `garrysmod/lua/includes/init.lua` to `_init.lua`
3. Symlink or copy repo's `lua/includes/init.lua` to `garrysmod/lua/includes/init.lua`
4. Put `gmsv_packuwus_linux.dll` binary module into `garrysmod/lua/bin` directory. [See instructions where you can get binary module](#getting-binary-module)

## Getting binary module

There's two ways how to get this module
1. <s>Download it from releases page</s> TODO xP
2. Compile it by yourself (see [binary building](#building-binary-module) section)

## Building binary module

**Windows binary module is not supported and never will.**

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Install nightly rust by running `rustup toolchain install nightly`
3. Add x86/i686 toolchain target by running `rustup target add i686-unknown-linux-gnu`
4. Run `cargo build --target=i686-unknown-linux-gnu --release`
5. Wait.  \~w\~  zZzZz...
6. Binary module is compiled to `./target/i686-unknown-linux-gnu/release/libgm_packuwus.so`
7. Rename it to `gmsv_packuwus_linux.dll`
8. Done! ^^
