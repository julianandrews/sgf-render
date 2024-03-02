# Sgf Render

![Continuous integration](https://github.com/julianandrews/sgf-render/workflows/Continuous%20integration/badge.svg)

![Cho Chikun Elementary, Problem 45](demo/prob45.svg).

CLI to generate SVG or PNG diagrams of Go games from
[SGF](https://www.red-bean.com/sgf/) format game records.

SVG output is clean and well labeled for easy re-styling or modification.

Supports [numbered
moves](https://raw.githubusercontent.com/julianandrews/sgf-render/master/demo/simple-numbered.svg),
[markup](https://raw.githubusercontent.com/julianandrews/sgf-render/master/demo/markup.svg),
[kifu output](https://raw.githubusercontent.com/julianandrews/sgf-render/master/demo/kifu.svg),
and
[several](https://raw.githubusercontent.com/julianandrews/sgf-render/master/demo/minimalist-numbered.svg)
[customizable](https://raw.githubusercontent.com/julianandrews/sgf-render/master/demo/wacky.svg)
[styles](https://raw.githubusercontent.com/julianandrews/sgf-render/master/demo/prob45-fancy.svg).

## Installation

Check the [releases](https://github.com/julianandrews/sgf-render/releases) page
on GitHub for pre-built binaries.

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
Usage: sgf-render [OPTIONS] [FILE]

Arguments:
  [FILE]  SGF file to render [default: read from stdin]

Options:
  -o, --outfile <FILE>           Output file [default: write to stdout]
  -f, --format <OUTPUT_FORMAT>   Output format [default: svg] [possible values: svg,
                                 png]
  -n, --node <PATH_SPEC>         Node to render. For simple use provide a number or
                                 `last` to render the last node. See the README for more
                                 detail
  -w, --width <WIDTH>            Width of the output image in pixels [default: 800]
  -s, --shrink-wrap              Draw only enough of the board to hold all the stones
                                 (with 1 space padding)
  -r, --range <RANGE>            Range to draw as a pair of corners (e.g. 'cc-ff')
      --style <STYLE>            Style to use [default: simple] [possible values:
                                 minimalist, fancy, simple]
      --custom-style <FILE>      Custom style `toml` file. Conflicts with '--style'. See
                                 the README for details
      --move-numbers[=<RANGE>]   Draw move numbers (may replace other markup)
      --move-numbers-from <NUM>  Number to start counting move numbers from (requires
                                 --move-numbers) [default: 1]
      --label-sides <SIDES>      Sides to draw position labels on [default: nw]
      --no-board-labels          Don't draw position labels
      --no-marks                 Don't draw SGF marks
      --no-triangles             Don't draw SGF triangles
      --no-circles               Don't draw SGF circles
      --no-squares               Don't draw SGF squares
      --no-selected              Don't draw SGF selected
      --no-dimmed                Don't draw SGF dimmed
      --no-labels                Don't draw SGF labels
      --no-lines                 Don't draw SGF lines
      --no-arrows                Don't draw SGF arrows
      --no-point-markup          Don't draw any markup on points
      --kifu                     Generate a kifu
  -h, --help                     Print help
  -V, --version                  Print version
```

If `FILE` isn't provided, `sgf-render` will read from stdin. If `--outfile`
isn't provided `sgf-render` will print the resulting SVG to stdout.

### Node selection

For the `--node` argument `PATH_SPEC` should be a comma-separated list of
steps.  A step can be a number which advances that many steps, 'v' followed by
a number which advances one step down the chosen variation, or `last` which
advances to the last node down the current variation.  Variations are
zero-indexed, so, for instance, `v0` is equivalent to `1`.

Note that the zeroth node in an SGF usually has no moves, but may have setup
(which is common for tsumego).  Nodes without moves (with commentary or
annotations) are possible, but uncommon.

Examples:

- `--node 0`: Show the root node (usually before the first move).
- `--node 7`: Show the 8th node (probably the 7th move) of the main variation.
- `--node last`: Show the last node of the main variation.
- `--node 5,v1,12`: Advance to the 6th node advance 1 step down the first
  (non-main) variation at that node, then advance 12 more steps. Show that
  node.
- `--node 5,v1,last`: Advance to the 6th node advance 1 step down the first
  (non-main) variation at that node, then advance to the last node. Show that
  node.

### Kifu mode

By default `sgf-render` generates diagrams designed to show the board position
at a single point in time. Captured stones are removed, and when using
`--move-numbers` only the last move number at a given point is displayed.
`--kifu` generates diagrams appropriate for use as a whole game record:

- move numbers are enabled,
- all other markup is disabled,
- stones are never removed from the board,
- if a stone would be placed on an existing stone an annotation is added
  instead, and
- `--node` defaults to `last` (can be overridden).

### Custom styles

You can use the `--custom-style` flag to specify a file with custom style
configuration in TOML format. As an example here's the style config for the
`simple` style:

```
line_color = "black"
line_width = 0.03
hoshi_radius = 0.09
background_fill = "#cfa87e"
label_color = "#6e5840"
black_stone_fill = "black"
white_stone_fill = "white"
black_stone_stroke = "black"
white_stone_stroke = "black"
markup_stroke_width = 0.1
black_stone_markup_color = "white"
white_stone_markup_color = "black"
empty_markup_color = "black"
black_stone_selected_color = "blue"
white_stone_selected_color = "blue"
empty_selected_color = "blue"
```

You can see a couple other examples in the source code package under
`resources/styles/`

## Contributing
Pull requests are welcome! For major changes, please open an issue first to
discuss what you would like to change.

Feature requests are also welcome! The goal is to make this a general purpose
sgf diagram generation tool. Just open an issue at
[GitHub](https://github.com/julianandrews/sgf-render/issues).
