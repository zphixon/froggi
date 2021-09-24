# Froggi specification

*version 0.0*

## Motivation

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

Request format: (offsets and lengths are in octets (also referred to as bytes))

|Offset|Length|Purpose|
|-|-|-|
|0|4|0xf0 0x9f 0x90 0xb8 froggi header 🐸|
|4|1|froggi version|
|5|1|request kind|
|6|16|client ID|
|22|2|request length = R|
|24|R|request|

Client ID is a UUID issued by a server if the client requests additional data
with request kind 0x2.

Request kinds:

* 0 - Plain old page. Don't send me any items or additional page expressions.
  I won't be talking to you again.
* 1 - Page with items, but no interactions. Don't send me any additional page
  expressions. I won't be talking to you again.
* 2 - Give me everything. I'll be in touch again for those extra page
  expressions.
* 14 - Here's some data. I'm eagerly awaiting your response.
* 15 - Unknown.

## Server

Response format: (offsets and lengths are in bytes)

|Offset|Length|Purpose|
|-|-|-|
|0|4|0xf0 0x9f 0x90 0xb8 froggi header 🐸|
|4|1|froggi version|
|5|1|response kind|
|6|16|client ID|
|22|4|total response length|
|26|4|page length = P|
|30|P|page|
|30+P|1|number of items|
|31+P|1|item kind|
|32+P|1|length of item name = N|
|33+P|N|item name|
|33+P+N|4|length of item = L|
|37+P+N|L|item|

Response kinds:

* 0 - Plain old page. I won't be sending any items or additional page
  expressions, so don't ask.
* 1 - Page with items. I won't be sending any additional page expressions.
* 2 - Additional page expressions. Feel free to do what you will with these.

Item kinds:

* 0 - Image. Up to the recipient to determine format.
* 15 - Error.

## Markup

### Page

A page in froggi markup includes optional page style, and an optional list of
expressions. The syntax of a page in froggi markup is essentially s-expressions.
They have an optional token after the left parenthesis, and then an expression,
possibly consisting of multiple sub-expressions, before the corresponding right
parenthesis.

### Page styles

A page style is a curly-braced list of parenthesized lists. The first token in
each list is the name of the style being defined, or the expression type the
style will apply to.

### Page expressions

A page expression consists of three parts:

* Expression type (optional, default is tall)
* Inline style (optional, in curly braces)
* Text xor children

#### Layout expression types

* `tall` - Child expressions will be laid out vertically, one after the other.
* `wide` - Child expressions will be laid out horizontally.
* `inline` - Child expressions will be laid out inline.

Page expressions with these types all follow the same syntax. See grammar.ebnf
for details.

#### Other expression types

* `#` - Page anchor. A named, non-visual reference point for more specific
  links or page expression insertion.
* `&` - Item reference. Insert the item by name at this spot in the page.
* `^` - Link. Provide a method of accessing the page, anchor, or file referred
  to by name or URL.
