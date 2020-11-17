# Sgf Render

CLI to generate SVG or PNG output from an SGF file.

SVG output is clean and well labeled for easy re-styling or modification.

![Cho Chikun Elementary, Problem 45](demo/prob45.svg).

## Installation

Check the [releases](https://github.com/julianandrews/sgf-render/releases) page
on GitHub for pre-built binaries.

Note that the windows, and linux-musl versions are compiled without PNG
support. If you need PNG support on a platform without it consider building
`sgf-render` yourself. Alternatively, you can always install
[ImageMagick](https://imagemagick.org/index.php) to convert the SVG into any
format you like.

If you have `cargo` installed, you can also install the package from crates.io:

```
$ cargo install sgf-render
```

## Building

If you have `git` and `cargo` installed you can also build from source:

```
$ git clone https://julianandrews/sgf-render
$ cd sgf-render
$ cargo build --release
$ ./target/release/sgf-render -h
```

## Usage

```
Usage: sgf-render [FILE] [options]

Options:
    -o, --outfile FILE  Output file. SVG and PNG formats supported.
    -n, --node-num NUM  Node number to render (default 1). Note that SGFs may
                        have nodes without moves.
    -w, --width WIDTH   Width of the output image in pixels (default 800)
    -s, --shrink-wrap   Draw only enough of the board to hold all the stones
                        (with 1 space padding)
    -r, --range RANGE   Range to draw as a pair of corners (e.g. 'cc-ff')
        --style STYLE   Style to use. One of 'default', 'simple' or
                        'minimalist'
        --move-numbers  Draw move numbers.
        --first-move-number NUM
                        First move number to draw if using --move-numbers
        --no-labels     Don't render labels on the diagram
    -h, --help          Display this help and exit
```

If `FILE` isn't provided, `sgf-render` will read from stdin. If `--outfile`
isn't provided `sgf-render` will print the resulting SVG to stdout.

## Contributing
Pull requests are welcome! For major changes, please open an issue first to
discuss what you would like to change.

Feature requests are also welcome! The goal is to make this a general purpose
sgf diagram generation tool. I have plans to add support for sgf markup
rendering, but motivation is low since I don't have any immediate use for it. A
feature request would get me moving! Just open an issue at
[GitHub](https://github.com/julianandrews/sgf-render/issues).
