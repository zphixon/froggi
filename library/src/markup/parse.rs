use super::ast::Item;

pub fn parse(data: &str, filename: String) -> Vec<Item<'_>> {
    let _tokens = super::scan::lex(data, filename);
    vec![]
}

#[cfg(test)]
mod test {
    #[test]
    fn sample() {
        let sample = include_str!("../../../sample_markdown.scm");
        super::super::scan::lex(sample, "../../sample_markdown.scm".into()).unwrap();
    }
}
