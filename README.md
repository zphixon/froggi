# Froggi protocol - [spec](https://github.com/zphixon/froggi/blob/main/spec.md)

This new protocol aims to be somewhere between gopher and http. Somewhat
inspired by gemini, and conceived due to gemini's unimpressive markup language.

Be warned, much of the readme below is subject to change and probably goes out
of date on every commit.

## Libraries

* Network
  * https://doc.rust-lang.org/stable/std/net/index.html
  * https://github.com/sfackler/rust-native-tls
  * https://github.com/ctz/rustls
  * https://github.com/tantivy-search/tantivy
* Client graphics
  * https://github.com/unicode-rs/unicode-segmentation
  * https://github.com/servo/unicode-bidi
  * https://crates.io/crates/harfbuzz_rs
  * https://github.com/redox-os/rusttype
  * https://github.com/servo/pathfinder
  * https://github.com/hecrj/iced
  * https://github.com/xi-editor/druid

## TODO

* possibly unify Page and Document types
  * parse would directly produce a Document rather than having to go through the
    intermediate Page syntax tree
* fix this inconsistent wording between page and document
  * also item is a terrible name
* switch to snafu for error types
* write *alllllll* the docs
  * [ ] protocol spec - the absolute minimum is there, we still need the fluffy
    bits that explain what each part of a request/response is and what
    clients/servers should do with them
  * [ ] library - again, MVP achieved but we still need more fluffy bits like
    usage
* write a markup validator
  * checks your pages for any broken links or object references
* server app
  * TLS by default
* client app
  * translate markup ast into layout tree
  * basically everything lol

### styling algorithm

1. width calculation: calculate how many pixels we have horizontally from the
   viewport size and screen DPI. then distribute this width evenly, or by
   ratio according to markup styles. text cannot make a box wider, only taller.

2. height calculation: text is broken into words (unicode_segmentation), shaped
   (harfbuzz-rs), rasterized (??), and stored for drawing later. each bit of
   text can be shaped independently since style changes can't cross box
   boundaries. add up word widths until it overflows the box width, then add
   the line height to the box height. keep going until we run out of text.

![Diagram](https://github.com/zphixon/froggi/blob/main/notes/display.svg)

## other ideas

* semantic color schemes for compatibility with system themes?
  * note, warning, quote
* unordered/ordered list
* links
  * same document jumps the view
  * other document opens that document
  * specify exactly where something links to

## unanswered questions
* input
  * single and multi-line text
  * file, button, radio, checkbox
  * client-side validation
* what to do if a word is too long for a box
  * hyphenization
  * truncation
* does fill make sense on a tall?
* responsiveness
  * squish boxes
  * make all non-inline boxes vertical

