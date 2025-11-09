# stele

stele is a lightweight macos gui clipboard utility built with rust and [gpui](https://www.gpui.rs/)

this is built for personal use and exploratory purposes - no guarantees can be made about the code quality

## quick start

```bash
# 1. install dependencies
rustup default stable                 # or nightly if needed

# 2. run the app in debug mode
cargo run

# 3. build an optimized binary
cargo run --release
```

when the binary is running:

1. copy any text as usual (`⌘c`).
2. hit `⌘⇧v` to toggle the stele panel.
3. click an entry to copy it back to the clipboard—the panel closes automatically.

## todo

[ ] add image support
[ ] add config settings
