# AsciiArena

*AsciiArena* is a terminal multiplayer deatchmatch game.
Choose an ascii character and use your arcade skills to be the last one in the arena!

<p align="center">
  <img src="https://drive.google.com/uc?export=view&id=1TMTNIbn09Ssh_e1VnhWEUhb5zYfRNyiw"/>
</p>

**Currently under development (only the menu is completed)**

The game is made in [rust][rust], and built on top of
[tui][tui] for rendering into the terminal user interface and
[message-io][message-io] for making the network connections.

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

- To launch the server:
    ```sh
    asciiarena server -p <number of players>
    ```

- To launch the client:
    ```sh
    asciiarena client
    ```

Both application modes has several CLI flags nad options to select
the host, ports, enable logs, etc...


<!-- Links here! -->
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[rust]: https://www.rust-lang.org/
[tui]: https://github.com/fdehau/tui-rs
[message-io]: https://github.com/lemunozm/message-io
