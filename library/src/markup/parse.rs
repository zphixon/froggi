use crate::FroggiError;

use super::ast::Item;

pub fn parse(data: &str) -> Result<Vec<Item<'_>>, FroggiError> {
    let tokens = super::scan::lex(data);
    println!("{:#?}", tokens);
    Ok(vec![])
}

#[cfg(test)]
mod test {
    #[test]
    fn sample() {
        let sample = include_str!("../../../server/pages/index.fml");
        super::super::scan::lex(sample).unwrap();
    }
}
