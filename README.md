# Infinite Online Minesweeper

It's minesweeper but online and infinite.

## How to build

There is a server and a client. Build both the server and the web client using `build.sh`.

```shell
build.sh
```

This will build the web client and put it in the `crates/server/static` directory, then build the server.
In development, the files will be served from the `crates/server/static` directory. In production, the files in this
directory are bundled into the executable in `target/release/sweeper-server`.