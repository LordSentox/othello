# Othello

An **online** implementation of the classic game of othello. You can create a server and play against your friends. **Offline mode** is currently unavailable, but will surely come back. Be aware, that this game is currently in ***early development*** stages and may not run to your satisfaction. Now that you are warned, the next part will explain how to compile and run the game, once you have aquired the source code.

### How do I run the game?
---
##### Prerequesites:
Othello is written in [rust](http://rust-lang.org) and uses the rust packet manager *cargo* to manage its dependencies and targets, so you will need to install both in order to compile and run this project. Also, you will need to install the C-Bindings for [sfml2](http://sfml-dev.org) on your computer, since this project relies on it for its graphics code. (If you are using a packet manager, look for either csml or maybe just sfml and they should pop up.)

##### Starting a server:
If you want to start a debug server, you can start one with:
```sh
cargo run --bin server
```
The port the server is going to listen to is specified in the server.toml. By default it will be 44942.

##### Starting a client and connecting:
If you know a server, or have started one yourself, you will want to connect to it. Currently, the Server address is managed in the ```client.toml```, which should be fairly straightforward to edit. Let it just be said, that the login name does not have to be provided in the ```client.toml```. In that case you can enter one once prompted by the client.
The client can again be started using cargo using:
```sh
cargo run --bin client
```

If you want to compile the game to run it without using cargo, you can use
```sh
cargo build --release
```
to build the release binaries which then can be found in the ```target/``` folder. You can then provide the port for the server to listen to as a command line argument. The data files will then need to go into the same directory as the executable.

##### Playing the game:
Finally, the fun part. Once you have started the game you will be greeted with a rather blank console. Just try entering ```help``` and the program will hopefully successfully talk you through from there.

### Contributing
---
Contributions are highly welcomed in any form. The game is in early development, so I am sure you will encounter a multitude of bugs or things you would like to change. If you have the spare time (after all, you're playing othello.. right?), please feel free to open an [issue](https://github.com/LordSentox/othello/issues/new) on the issue tracker, or if getting to work on it yourself is more your style, please don't hesitate to create a pull request.
