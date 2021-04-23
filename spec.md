# Froggi specification

*version 0.0*

### Motivation

The web is easily one of the best things that we as a species have done, and
despite its overwhelming complexity, it continues to be a place of wonder. But
the most prevalent trend since its original incarnation as a network of
scientific documents pioneered at CERN in 1989 has been to slowly expand its set
of features. While this has generally been a good thing (style sheets,
scripting, websockets, DASH) the general openness of the web has resulted in the
inclusion of some less savory parts. It's super cool that you can buy stuff
online and have it show up at your door, watch something happen in real time
from halfway around the world, or talk to almost anyone anywhere with relatively
little resistance (relative to 50 years ago, at least), but the slow erosion of
democracy, the rampant spread of misinformation, and the commercialization of
humanity's greatest achievement have understandably left a sour taste in many
people's mouths. Given the recent resurgence in gopher, and the creation of
gemini, it's clear that people are itching for something reminiscent of the Wild
West Web.

## Compatibility

Froggi data is always utf8, aside from bonus items which have no specified
encoding. Numbers are little-endian, except for client IDs, which are
big-endian.

## Client

Request format: (offsets and lengths are in bytes)

|Offset|Length|Purpose|
|-|-|-|
|0|1|froggi version|
|1|1|request kind|
|2|16|client ID|
|18|2|request length = R|
|20|R|request|

Client ID is a UUID issued by a server if the client requests additional data
with request kind 0x2.

Request kinds:

* 0x0 = page
* 0x1 = page with no items
* 0x2 = additional

## Server

Response format: (offsets and lengths are in bytes)

|Offset|Length|Purpose|
|-|-|-|
|0|1|froggi version|
|1|1|response kind|
|2|16|client ID|
|18|4|total response length|
|22|4|page length = P|
|26|P|page|
|26+P|1|number of items|
|27+P|1|item kind|
|28+P|1|length of item name = N|
|29+P|N|item name|
|29+P+N|4|length of item = L|
|33+P+N|L|item|

Response kinds:

* 0x0 = page
* 0x1 = page with no items
* 0x2 = embed

Item kinds:

* 0x0 = png
* 0x1 = jpg
* 0x2 = gif

## Markup

### Page

A page in froggi markup includes optional page style, and an optional list of
items. Every page is as if it was in an implicit `:vbox` item.

### Items

Items in a froggi markup page consist of four parts, three of which are
optional:

* (optional) the builtin name
* (optional) the user-defined style
* (optional) inline styles
* text or children

The syntax of an item looks like:

`(text {user-style sans (bg "303030")} "Hello world in sans-serif!")`

The item has a built-in name, `text`, which is implied for all items that do not
specify one. The item has a user-defined style `user-style`, which is specified
in the top-level page style item. The item has inline styling
`{sans (bg "303030")}`, with a style that takes an argument `(bg "303030")`, and
a style that does not `sans`. The item has text.

### Page style

A style item in the page-level style includes a built-in name or a user-defined
style name, followed by all styles that will be applied to items with that
user-defined style name or built-in item name.

```
{(text italic)
 (user-style (fg "fff8dc"))}
```

## Built-in item names

* `text`
* `box`
* `vbox`

## Built-in style names

* Font styles with no args:
  * `serif`
  * `sans`
  * `mono`
  * `italic`
  * `underline`
  * `bold`
  * `strike`
* Font styles with args:
  * `(fg "color")`
  * `(size "1em")`
* Other styles with args:
  * `(bg "color")`
  * `(fill "ratio")`

