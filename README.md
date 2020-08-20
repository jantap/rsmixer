# RsMixer

![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/jantap/rsmixer)

RsMixer is a PulseAudio volume mixer written in rust

## Features

- monitors displaying current volume
- applications using outputs displayed in a nested tree structure for easier viewing
- changing card settings
- all the basic stuff you expect a volume mixer to do

## Installation

You can install RsMixer through cargo:

```
cargo install rsmixer
```

or by manually building it:

```
git clone https://github.com/jantap/rsmixer.git
cargo install --path ./rsmixer
```

## Known issues

Context menu doesn't handle the situation when the terminal window can't fit the contents of the menu

## License

[MIT](https://choosealicense.com/licenses/mit/)
