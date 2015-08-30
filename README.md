# webdis in Rust

[webdis](http://webd.is/) is a HTTP interface for Redis, [@markuman](https://github.com/markuman) reimplemented it [in a few lines of Lua](https://github.com/markuman/tinywebdis/blob/master/turbowebdis.lua).
I used Rust now.

This code is highly experimental, uses at least 5 `unrwap()`s and is definitely not finished (or will ever be).

But it works.

## Running

```
cargo run --release
```

It listens on port 3000 on localhost, so you can now just send commands:

```
$ http://localhost:3000/set/foo/ok
{"data":"OK"}
$ http://localhost:3000/incr/a
{"data":1}
$ http://localhost:3000/incrby/a/12
{"data":13}
$ http://localhost:3000/del/z
{"data":0}
$ http://localhost:3000/mget/foo/a/z
{"data":["ok","13",null]}
```
