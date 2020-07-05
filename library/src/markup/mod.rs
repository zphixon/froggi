pub mod parse;
pub mod scan;

use scan::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub struct Page<'a> {
    pub page_styles: Vec<PageStyle<'a>>,
    pub items: Vec<PageItem<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PageStyle<'a> {
    pub selector: Token<'a>,
    pub styles: Vec<InlineStyle<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PageItem<'a> {
    pub builtin: Token<'a>,
    pub inline_styles: Vec<InlineStyle<'a>>,
    pub payload: ItemPayload<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ItemPayload<'a> {
    Text {
        text: Vec<Token<'a>>,
    },
    Children {
        children: Vec<PageItem<'a>>,
        line: usize,
    },
    Link {
        link: Token<'a>,
        text: Vec<Token<'a>>,
    },
    Blob {
        name: Token<'a>,
        alt: Vec<Token<'a>>,
    },
    Anchor {
        anchor: Token<'a>,
    },
}

#[derive(Debug, PartialEq)]
pub enum InlineStyle<'a> {
    Mono { token: Token<'a> },
    Serif { token: Token<'a> },
    Sans { token: Token<'a> },
    Bold { token: Token<'a> },
    Italic { token: Token<'a> },
    Underline { token: Token<'a> },
    Strike { token: Token<'a> },
    Fg { token: Token<'a>, arg: Token<'a> },
    Bg { token: Token<'a>, arg: Token<'a> },
    Fill { token: Token<'a>, arg: Token<'a> },
    Size { token: Token<'a>, arg: Token<'a> },
    UserDefined { token: Token<'a> },
}

pub fn to_html(page: &Page) -> String {
    let mut html = String::from(
        r#"
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf8">
    <style>
div {
    display: flex;
}
div > * {
    flex-basis: 0;
    flex-grow: 1;
    padding: 3px 3px 7px 3px;
}
body {
    max-width: 850px;
    margin: 0 auto;
    float: none;
}
"#,
    );

    for page_style in &page.page_styles {
        match page_style.selector.kind() {
            TokenKind::Identifier => {
                html.push_str(&format!(".{} {{\n", page_style.selector.lexeme()));
            }

            TokenKind::Blob => {
                html.push_str("img {\n");
            }

            TokenKind::Link => {
                html.push_str("a {\n");
            }

            TokenKind::Anchor => {
                // n/a
            }

            TokenKind::Box | TokenKind::VBox => {
                html.push_str("div {\n");
            }

            TokenKind::Text | TokenKind::ImplicitText | TokenKind::Inline => {
                html.push_str("span {\n");
            }

            _ => unreachable!(),
        }

        for inline_style in &page_style.styles {
            html.push_str(&format!("    {}\n", inline_style_to_html(inline_style)));
        }
        html.push_str("}\n");
    }
    html.push_str("    </style>\n  </head>\n  <body>\n");

    for item in &page.items {
        html.push_str(&page_item_to_html(item, false));
    }

    html.push_str(
        "  <script>
    if (window.location.hash) {
      var elt = document.getElementById(
        window.location.hash.substring(1)
      );
      elt.scrollIntoView(true);
    }\n  </script>\n",
    );

    html.push_str("  </body>\n</html>\n");

    html
}

fn page_item_to_html(item: &PageItem, child_of_inline: bool) -> String {
    let mut html = String::new();
    match &item.payload {
        ItemPayload::Text { text } => {
            html.push_str("<span");

            if !item.inline_styles.is_empty() {
                html.push_str(&style_list_to_html(item));
            }

            html.push_str(">");
            html.push_str(&text.iter().fold(String::new(), |acc, next| {
                format!("{}{}", acc, next.lexeme())
            }));

            html.push_str(&format!(
                "</span>{} <!-- text {} -->\n",
                if child_of_inline { "" } else { "<br>" },
                item.builtin.line(),
            ));
        }

        ItemPayload::Children { children, .. } => {
            let tag = if item.builtin.kind() == TokenKind::Inline {
                "span"
            } else {
                "div"
            };

            html.push_str(&format!("<{}", tag));

            if !item.inline_styles.is_empty() {
                html.push_str(&style_list_to_html(item));
            }

            if item.builtin.kind() == TokenKind::VBox {
                html.push_str(" style=\"flex-direction: column;\"");
            }

            html.push_str(&format!(
                "> <!-- {} {} -->\n",
                item.builtin.lexeme(),
                item.builtin.line()
            ));

            for child in children {
                html.push_str(&format!(
                    "{}",
                    page_item_to_html(child, item.builtin.kind() == TokenKind::Inline)
                ));
            }

            html.push_str(&format!("</{}>", tag));
            if item.builtin.kind() == TokenKind::VBox || item.builtin.kind() == TokenKind::Inline {
                html.push_str("<br>\n");
            } else {
                html.push_str("\n");
            }
        }

        ItemPayload::Link { link, text } => {
            html.push_str("<div");

            if !item.inline_styles.is_empty() {
                html.push_str(&style_list_to_html(item));
            }

            html.push_str(">");
            html.push_str(&format!("<a href=\"{}\">", link.lexeme()));
            if !text.is_empty() {
                html.push_str(&text.iter().fold(String::new(), |acc, next| {
                    format!("{}{}", acc, next.lexeme())
                }));
            } else {
                html.push_str(link.lexeme());
            }

            html.push_str("</a></div>\n");
        }

        ItemPayload::Blob { name, alt } => {
            // TODO: style
            // <embed>? image type?
            html.push_str(&format!("<img src=\"{}\"", name.lexeme()));
            if !alt.is_empty() {
                html.push_str(" alt=\"");
                html.push_str(&alt.iter().fold(String::new(), |acc, next| {
                    format!("{}{}", acc, next.lexeme())
                }));
                html.push_str("\"");
            }
            html.push_str(">\n");
        }

        ItemPayload::Anchor { anchor } => {
            html.push_str(&format!(
                "<div id=\"{}\" style=\"display:hidden;\"></div>\n",
                anchor.lexeme()
            ));
        }
    }

    html
}

fn style_list_to_html(item: &PageItem) -> String {
    let mut html = String::new();
    let mut classes = Vec::new();
    let mut styles = Vec::new();

    for style in &item.inline_styles {
        match style {
            InlineStyle::UserDefined { token } => {
                classes.push(token.lexeme());
            }
            _ => styles.push(inline_style_to_html(style)),
        }
    }

    if !classes.is_empty() {
        html.push_str(" class=\"");
        for (i, class) in classes.iter().enumerate() {
            html.push_str(class);
            if i + 1 < classes.len() {
                html.push(' ');
            }
        }
        html.push_str("\"");
    }

    if !styles.is_empty() {
        html.push_str(" style=\"");

        for (i, style) in styles.iter().enumerate() {
            html.push_str(style);
            if i + 1 < styles.len() {
                html.push(' ');
            }
        }

        html.push_str("\"");
    }

    html
}

fn inline_style_to_html(style: &InlineStyle) -> String {
    match style {
        InlineStyle::Mono { .. } => String::from("font-family: monospace;"),
        InlineStyle::Serif { .. } => String::from("font-family: serif;"),
        InlineStyle::Sans { .. } => String::from("font-family: sans-serif;"),
        InlineStyle::Bold { .. } => String::from("font-weight: bold;"),
        InlineStyle::Italic { .. } => String::from("font-style: italic;"),
        InlineStyle::Underline { .. } => String::from("text-decoration: underline;"),
        InlineStyle::Strike { .. } => String::from("text-decoration: line-through;"),
        InlineStyle::Fg { arg, .. } => format!("color: #{};", arg.lexeme()),
        InlineStyle::Bg { arg, .. } => format!("background-color: #{};", arg.lexeme()),
        InlineStyle::Fill { .. } => String::from("/* TODO: fill */"),
        InlineStyle::Size { arg, .. } => format!("font-size: {}px;", arg.lexeme()),
        InlineStyle::UserDefined { .. } => unreachable!(),
    }
}
