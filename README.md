# temi: TUI for Lemmy

`temi` is a command-line, lightweight TUI client for Lemmy.

The client is currently very experimental, and fairly bare-bones. It only acts as a reader, for example.

It is still under heavy development, with many more features to come.

## Supported features

- read Lemmy posts and comments from the terminal
  - `LEMMY_INSTANCE="https://your.favorite.instance" cargo run`
    - choosing your instance will likely change to a config/command-line argument

## Planned features

- login to a Lemmy instance
- post something
- view images directly in the TUI
  - WIP: [libsixel-rs](https://github.com/rmsyn/libsixel-rs) will allow directly viewing images in the terminal
  - [sixel-rs](https://github.com/AdnoC/sixel-rs) could also work, but I rather not include a C dependency
  - requires Sixel support in the terminal
  - [Kitty Graphic Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) is another option to pursue
- view posts from federated instances
- support the full Lemmy API surface
