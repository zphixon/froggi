# froggi specification

*version 0.0*

### motivation

the web is easily one of the best things that we as a species have done, and
despite its overwhelming complexity, it continues to be a place of wonder. but
the most prevalent trend since its original incarnation as a network of
scientific documents pioneered at CERN in 1989 has been to slowly expand its
set of features. while this has generally been a good thing (style sheets,
scripting, websockets, DASH) the general openness of the web has resulted in
the inclusion of some less savory parts. while it's super cool that you can buy
stuff online and have it show up at your door, watch something happen in real
time from halfway around the world, or talk to almost anyone anywhere with
relatively little resistance (relative to 50 years ago, at least), the slow
erosion of democracy, the rampant spread of misinformation, and the commercialization
of humanity's greatest achievement have understandably left a sour taste in
many people's mouths. given the recent resurgence in gopher, and the creation
of gemini, it's clear that people are itching for something reminiscent of the
Wild West Web.

## compatibility

froggi data is always utf8, aside from bonus items which have no specified encoding. numbers
are little-endian.

## client

request format: (offsets and lengths are in bytes)

|offset|length|purpose|
|-|-|-|
|0|1|froggi version|
|1|1|request kind|
|2|2|request length = R|
|4|R|request|

request kinds:

* 0x0 = page
* 0x1 = page with no items

## server

response format: (offsets and lengths are in bytes)

|offset|length|purpose|
|-|-|-|
|0|1|froggi version|
|1|1|response kind|
|2|4|total response length|
|6|4|page length = P|
|12|P|page|
|12+P|1|number of items|
|13+P|1|item kind|
|14+P|1|length of item name = N|
|15+P|N|item name|
|15+P+N|4|length of item = L|
|19+P+N|L|item|

## markup

### page

a page in froggi markup includes optional page style, and an optional list of
items. every page is as if it was in an implicit `:vbox` item.

### items

items in a froggi markup page consist of four parts, three of which are
optional:

* (optional) the builtin name
* (optional) the user-defined style
* (optional) inline styles
* text or children

the syntax of an item looks like:

`(text {user-style sans (bg "303030")} "Hello world in sans-serif!")`

the item has a built-in name, `text`, which is implied for all items that do
not specify one. the item has a user-defined style `user-style`, which is
specified in the top-level page style item. the item has inline styling
`{sans (bg "303030")}`, with a style that takes an argument `(bg "303030")`,
and a style that does not `sans`. the item has text.

### page style

a style item in the page-level style includes a built-in name or a user-defined style
name, followed by all styles that will be applied to items with that user-defined
style name or built-in item name.

```
{(text italic)
 (user-style (fg "fff8dc"))}
```

## built-in item names

* `text`
* `box`
* `vbox`

## built-in style names

* font styles with no args:
  * `serif`
  * `sans`
  * `mono`
  * `italic`
  * `underline`
  * `bold`
  * `strike`
* font styles with args:
  * `(fg "color")`
  * `(size "1em")`
* other styles with args:
  * `(bg "color")`
  * `(fill "ratio")`

