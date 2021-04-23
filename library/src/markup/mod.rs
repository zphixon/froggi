pub mod page;
pub mod parse;
pub mod scan;

use scan::{Token, TokenKind};

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Page<'a> {
    pub styles: PageStyles<'a>,
    pub expressions: Vec<PageExpression<'a>>,
}

pub type PageStyles<'a> = HashMap<Token<'a>, Vec<InlineStyle<'a>>>;

#[derive(Debug, PartialEq)]
pub struct PageExpression<'a> {
    pub builtin: Token<'a>,
    pub styles: Vec<InlineStyle<'a>>,
    pub payload: ExpressionPayload<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionPayload<'a> {
    Text {
        text: Vec<Token<'a>>,
    },
    Children {
        children: Vec<PageExpression<'a>>,
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
    Fg { token: Token<'a>, arg: (u8, u8, u8) },
    Bg { token: Token<'a>, arg: (u8, u8, u8) },
    Fill { token: Token<'a>, arg: f32 },
    Size { token: Token<'a>, arg: usize },
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

    for (selector, page_styles) in &page.styles {
        match selector.kind() {
            TokenKind::Identifier => {
                html.push_str(&format!(".{} {{\n", selector.lexeme()));
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

            TokenKind::Wide | TokenKind::Tall => {
                html.push_str("div {\n");
            }

            TokenKind::Text | TokenKind::Inline => {
                html.push_str("span {\n");
            }

            _ => unreachable!(),
        }

        for inline_style in page_styles {
            html.push_str(&format!("    {}\n", inline_style_to_html(inline_style)));
        }
        html.push_str("}\n");
    }
    html.push_str("    </style>\n  </head>\n  <body>\n");

    for expression in &page.expressions {
        html.push_str(&page_expression_to_html(expression, false));
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

fn page_expression_to_html(expression: &PageExpression, child_of_inline: bool) -> String {
    let mut html = String::new();
    let not_flex_column = false;
    match &expression.payload {
        ExpressionPayload::Text { text } => {
            html.push_str("<span");

            if !expression.styles.is_empty() {
                html.push_str(&style_list_to_html(expression, not_flex_column));
            }

            html.push_str(">");
            html.push_str(&text.iter().fold(String::new(), |acc, next| {
                format!("{}{}", acc, next.lexeme())
            }));

            html.push_str(&format!(
                "</span>{} <!-- text {} -->\n",
                if child_of_inline { "" } else { "<br>" },
                expression.builtin.line(),
            ));
        }

        ExpressionPayload::Children { children, .. } => {
            let is_vertical = expression.builtin.kind() == TokenKind::Tall;
            let is_inline = expression.builtin.kind() == TokenKind::Inline;
            let tag = if is_inline { "span" } else { "div" };

            html.push_str(&format!("<{}", tag));

            html.push_str(&style_list_to_html(expression, is_vertical));

            html.push_str(&format!(
                "> <!-- {} {} -->\n",
                expression.builtin.lexeme(),
                expression.builtin.line()
            ));

            for child in children {
                html.push_str(&format!("{}", page_expression_to_html(child, is_inline)));
                if is_vertical {
                    html.push_str("<br>");
                }
            }

            html.push_str(&format!("</{}>", tag));
            if is_vertical || is_inline {
                html.push_str("<br>\n");
            } else {
                html.push_str("\n");
            }
        }

        ExpressionPayload::Link { link, text } => {
            html.push_str("<div");

            if !expression.styles.is_empty() {
                html.push_str(&style_list_to_html(expression, not_flex_column));
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

        ExpressionPayload::Blob { name, alt } => {
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

        ExpressionPayload::Anchor { anchor } => {
            html.push_str(&format!(
                "<div id=\"{}\" style=\"display:hidden;\"></div>\n",
                anchor.lexeme()
            ));
        }
    }

    html
}

fn style_list_to_html(expression: &PageExpression, flex_column: bool) -> String {
    let mut html = String::new();
    let mut classes = Vec::new();
    let mut styles = Vec::new();

    for style in &expression.styles {
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

        if flex_column {
            html.push_str("flex-direction: column;");
        }

        for (i, style) in styles.iter().enumerate() {
            html.push_str(style);
            if i + 1 < styles.len() {
                html.push(' ');
            }
        }

        html.push_str("\"");
    } else if flex_column {
        html.push_str(" style=\"flex-direction: column;\"");
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
        InlineStyle::Fg { arg, .. } => format!("color: #{:02x}{:02x}{:02x};", arg.0, arg.1, arg.2,),
        InlineStyle::Bg { arg, .. } => format!(
            "background-color: #{:02x}{:02x}{:02x};",
            arg.0, arg.1, arg.2,
        ),
        InlineStyle::Fill { arg, .. } => String::from(format!("flex-grow: {};", arg)),
        InlineStyle::Size { arg, .. } => format!("font-size: {}px;", arg),
        InlineStyle::UserDefined { .. } => unreachable!(),
    }
}
