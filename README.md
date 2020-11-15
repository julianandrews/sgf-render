# Sgf Render

CLI to generate `.svg` or `.png` output from an `.sgf` file.

SVG output is clean and well labeled for easy re-styling or modification.

![Cho Chikun Elementary, Problem 45](data/problem45.svg).

## Installation

TODO: Add CI to build binaries, and link here.

## Usage

```
Usage: sgf-render [FILE] [options]

Options:
    -o, --outfile FILE  Output file. SVG and PNG formats supported.
    -m, --move-num NUM  Move number to render (default 1)
    -w, --width WIDTH   Width of the output image in pixels (default 800)
    -s, --shrink-wrap   Draw only enough of the board to hold all the stones
                        (with 1 space padding)
    -r, --range RANGE   Range to draw as a pair of corners (e.g. 'cc-ff')
        --no-labels     Don't render labels on the diagram
    -h, --help          Display this help and exit
```

## Contributing
Pull requests are welcome! For major changes, please open an issue first to
discuss what you would like to change.

Feature requests are also welcome. The goal is to make this a general purpose
sgf diagram generation tool. I have plans to add support for sgf markup
rendering, but motivation is low since I don't have any immediate use for it. A
feature request would get me moving! Just open an issue at
[GitHub](https://github.com/julianandrews/sgf-render/issues).

