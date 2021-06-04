Key Configurator
================

You know what's really annoying about Linux?  Control+C and Control+V will copy and paste in every single application except the terminal, since Control+C is reserved for a SIGTERM.  OSX has much more intuitive key bindings for this where copy and paste are Super+C and Super+V, which leaves Control+C free for the terminal.  On top of that, not all Linux applications allow you to rebind the copy+paste commands.

I often switch between OSX and Linux so I'm always mistyping.

This project is a simple Rust program that intercepts key events and translates certain key combinations to the `uinput` virtual device.

Lots of things are hardcoded for now (like the file descriptor name of my laptop's keyboard -- heh) but one day I'd like to make a flexible / configurable keyboard configurator like Karabiner on OSX.

# Requirements
* libevdev

# Installation
## Ubuntu
```
sudo apt install libevdev-dev
```

# Usage

```
cargo run
```
