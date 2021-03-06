use super::super::core::{Result, Signature, Value};

pure_function!(
    LIST,
    Signature::new(vec![], "rest".into(), vec![], "".into()),
    list
);

async fn list(vs: Vec<Value>) -> Result {
    Ok(vs[0].clone())
}

#[cfg(test)]
mod test {
    use futures::stable::block_on_stable;

    use super::*;

    use super::super::super::core::functions::EQUAL;
    use super::super::super::core::{app, papp, Arguments, Expansion, List};

    #[test]
    fn list() {
        for (a, l) in vec![
            (Arguments::positionals(&[]), List::Empty),
            (
                Arguments::positionals(&[42.into()]),
                List::new(&[42.into()]),
            ),
            (
                Arguments::new(&[Expansion::Expanded(List::new(&[42.into()]).into())], &[]),
                List::new(&[42.into()]),
            ),
            (
                Arguments::new(
                    &[
                        Expansion::Unexpanded(1.into()),
                        Expansion::Expanded(List::new(&[2.into(), 3.into()]).into()),
                        Expansion::Unexpanded(4.into()),
                        Expansion::Expanded(List::new(&[5.into(), 6.into()]).into()),
                    ],
                    &[],
                ),
                List::new(&[1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into()]),
            ),
        ] {
            println!(
                "{}",
                block_on_stable(app(LIST.clone(), a.clone()).to_string()).unwrap()
            );

            assert!(
                block_on_stable(papp(EQUAL.clone(), &[app(LIST.clone(), a), l.into()]).boolean())
                    .unwrap(),
            );
        }
    }
}
