# kakure (隠れ) -- A chat application running on the DNS protocol

This repository contains the code for `kakure`, a simple proof-of-concept chat application that uses the DNS protocol as carrier for messages.

[![asciicast](https://asciinema.org/a/Ov6HVpkPQDeqebTcOn4B3shO4.svg)](https://asciinema.org/a/Ov6HVpkPQDeqebTcOn4B3shO4)

## How does it work?

I went into detail about this project and how it works [in this blog post](https://dummyco.de/building-a-chat-protocol-on-top-of-dns/).

## What's that name?

I started this project when I had been learning Japanese for about 2 years. It's a fascinating language. And the japanese word 隠れ translates to _concealed, hidden_. I guess that describes pretty good what this does. :shrug:

## Building the project

Just grab a copy of the source and install [Rust](https://rustup.rs). Then go for
```bash
cargo build
```

The `--help` command tells you how to modify the source and destination ports.

## Licensing?

This project is licensed under GPLv3.
