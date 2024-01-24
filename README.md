# Beepy games

A collection of games for the [Beepy](https://beepy.sqfmi.com/). (not yet really, it's just one now)

## How to run

You can find compiled binaries on the [releases page](https://github.com/chamik/beepy-games/releases).

To run these games you will need to add yourself to the `video` group:
```sh
$ sudo usermod -aG video "$USER"
```

And then give yourself write access to `/sys/class/vtconsole/vtcon1/bind`*. This can be done in several ways:
1. Running the game as root (one-time, easiest)
2. Executing `sudo chmod o+w /sys/class/vtconsole/vtcon1/bind` (lasts until reboot)
3. Writing a udev rule (going to write down how soon™) (permanent)

*This is so the program can unbind the virtual console to avoid weird flickering issues.

## How co compile

```sh
$ cross build --target arm-unknown-linux-gnueabihf --release --bin <game name>
```