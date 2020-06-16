use crate::FroggiError;

use super::ast::Item;

pub fn parse(data: &str) -> Result<Vec<Item<'_>>, FroggiError> {
    let tokens = super::scan::lex(data);
    Ok(vec![])
}

#[cfg(test)]
mod test {
    #[test]
    fn sample() {
        let sample = include_str!("../../../sample_markdown.scm");
        super::super::scan::lex(sample).unwrap();
    }
}
