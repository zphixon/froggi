; lowercase-name - builtin element with its own styling rules
; UppercaseName - user-defined element
; $serif - text choices
; #c00c00 - color
; %32 - fill percent, or how much space to take up vertically or horizontally

(page
  "Page title"
  (txt $serif #303030 $indent)
  (QuoteBox #FFF8DC)
  (QuoteText $no-indent $sans #606060)
  (Footnote #757575 $sans $italic))

; hbox adds elements to it horizontally in the top left of each sub-element
(hbox (h1 "Lorem ipsum example") (img header.jpg))

; txt is simple text, left-justified and beginning-aligned
(txt "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod"
     "tempor incididunt ut labore et dolore magna aliqua.")

; QuoteBox is a style selector
(hbox QuoteBox
  (txt QuoteText "Contrary to popular belief, Lorem Ipsum is not simly random text. It has roots in a piece of classical Latin literature")
  (txt QuoteText "from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in")
  ; normal box elements are inserted vertically
  (box
    (txt QuoteText "Virginia, looked up one of the more obscure Latin")
    (txt Footnote %20 "from https://www.lipsum.com/")))

(txt "But I must explain to you how all this mistaken idea of denouncing pleasure and praising pain was born and I will give you a complete account"
   "of the system, and expound the actual teachings of the great explorer of the truth, the master-builder of human happiness.")

