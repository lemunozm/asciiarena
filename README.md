# AsciiArena

*AsciiArena* is a terminal multiplayer deatchmatch game.
Choose an ascii character and be the last one in the arena using your arcade skills!

The game is made in [rust][rust], and built on top of
[tui][tui] for render into the terminal user interface and
[message-io][message-io] to make the network connections.

**Currently under development**

## Install it
*AsciiArena* is multiplatform, you can install it in Linux, MacOS or Windows.

You can use the [cargo][cargo] package manager in order to install it:
```
$ cargo install asciiarena
```
If you have `~/.cargo/bin` in your PATH (or similar in your OS), you will be able to use *asciiarena* everywhere in your computer!

Of course, you can download the repository and compile it by yourself using `cargo`.

## Try it!
*AsciiArena* application contains both the server and the client.

- To lunch the server:
    ```sh
    asciiarena server -p <number of players>
    ```

- To lunch the client:
    ```sh
    asciiarena client
    ```

Both application modes has several CLI fetures to select the host, ports, enable logs, etc...


<!-- Links here! -->
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[rust]: https://www.rust-lang.org/
[tui]: https://github.com/fdehau/tui-rs
[message-io]: https://github.com/lemunozm/message-io
