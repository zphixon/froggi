# froggi protocol

this new protocol aims to be somewhere between gopher and http. somewhat inspired
by gemini, and conceived due to its unimpressive markup language.

## libraries

* network
  * https://doc.rust-lang.org/stable/std/net/index.html
  * https://github.com/sfackler/rust-native-tls
  * https://github.com/ctz/rustls
  * https://github.com/tantivy-search/tantivy
* client graphics
  * https://github.com/unicode-rs/unicode-segmentation
  * https://github.com/servo/unicode-bidi
  * https://crates.io/crates/harfbuzz_rs
  * https://github.com/redox-os/rusttype
  * https://github.com/servo/pathfinder
  * https://github.com/hecrj/iced
  * https://github.com/xi-editor/druid

## markup

S-expression-based markup
* layout elements include `box`, `hbox`, `page`
* `lowercase` names are built-in styles. includes `txt`, `h1`, `img`, `link`, `page`, etc
* `Uppercase` names are user-defined styles

## styling

* goes after page title in the page item
* font styling with `$`: `$serif`, `$sans`, `$indent`, `$italic`, `$bold`, `$underline`
* `!` to negate styles, eg `!UserStyle`
* vertical and horizontal fill
  * `(hbox (item1) (item2 %20)`: item2 will have 20% the width of the surrounding `hbox`

### styling algorithm

1. width calculation
  * calculate how many pixels we have horizontally from viewport size and screen DPI.
  * calculate the required width of our content
    * shape each text item (harfbuzz). the longest word of a text item is the required width
    * the width of an image after any scaling
  * distribute the maximum of required and available pixels among horizontal items:
    * equally, or percentage-wise.
2. height calculation...

![diagram](https://github.com/zphixon/unnamed_protocol/blob/master/notes/display.svg)

## other ideas

* add flag byte?
  * whether or not the client wants images (would be useful for crawlers or search indexing)
* semantic color schemes for compatibility with system themes?
  * note, warning, quote
* unordered/ordered list
* paragraphs, indented or not
* links
  * same document jumps the view
  * other document opens that document
  * specify exactly where something links to
* input?
  * single and multi-line text
  * file?, button?, radio?, checkbox?, validation?

