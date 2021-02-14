# RsMixer

![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/jantap/rsmixer)
![Crates.io tag](https://img.shields.io/crates/v/rsmixer)

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

or if you're an Arch user, you can install it from AUR:

```
git clone https://aur.archlinux.org/rsmixer.git
cd rsmixer
makepkg -si
```

## Usage

Application screen is divided into 3 pages: Output, Input and Cards. Output combines PulseAudio sinks and sink inputs (if you don't know much about pulseaudio - basically sinks/sources are speakers/microphones, sink inputs/source outputs are audio streams from applications, for outputing and inputing sound respectively) into one tree-like view, that makes it easy to see which device every app uses.

All keybindings are configurable through `~/.config/rsmixer/rsmixer.toml`. [Changing keybindings][changing keybindings] for more info.

Default keybindings:

- j,k - move between entries
- h, l, H, L - change volume
- 1, 2, 3 - open outputs, inputs, and cards respectively
- enter - open context menu

## Changing keybindings

In `~/.config/rsmixer/rsmixer.toml` you will find a section `[bindings]`. There you will find a list of default keybindings.

All keybindings look one of these:

```
q = ['exit']
"shift+tab" = ['cycle_pages_backward']
right = ['raise_volume(5)']
```

Basically `key = ArrayOf(action)`. Key is either:

- a char
- a special key. [Special keys supported](special_keys.md) (if anything is missing just create an issue)
- a key combination, with plus signs between keys (one or more of shift, ctrl, alt and and a char/special key, seperated by plus signs)

When that key/key combination gets pressed rsmixer performs an action assigned to that keybinding. [Possible actions](actions.md)

## License

[MIT](https://choosealicense.com/licenses/mit/)
