use std::str::FromStr;

use pest::iterators::Pair;
use pest::Parser;

use super::super::ast::{
    Arguments, DefFunction, Effect, Expansion, Expression, HalfSignature, Import, InnerStatement,
    KeywordArgument, LetVariable, Module, OptionalParameter, Signature, Statement,
};

use super::error::ParsingError;

const _GRAMMAR: &'static str = include_str!("grammer.pest");

#[derive(Parser)]
#[grammar = "parse/grammer.pest"]
struct LanguageParser;

pub fn main_module(s: &str) -> Result<Module, ParsingError> {
    module(Rule::main_module, s)
}

pub fn sub_module(s: &str) -> Result<Module, ParsingError> {
    module(Rule::sub_module, s)
}

fn module(r: Rule, s: &str) -> Result<Module, ParsingError> {
    let mut is = vec![];
    let mut ss = vec![];

    let p = LanguageParser::parse(r, s)?.next().unwrap();

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::import => is.push(import(p)),
            Rule::statement => ss.push(statement(p)),
            Rule::inner_statement => ss.push(statement(p)),
            _ => unreachable!(),
        }
    }

    Ok(Module::new(is, ss))
}

fn import(p: Pair<Rule>) -> Import {
    let s = p.into_inner().next().unwrap().as_str();
    Import::new(s[1..(s.len() - 1)].into())
}

fn statement(p: Pair<Rule>) -> Statement {
    let p = p.into_inner().next().unwrap();

    match p.as_rule() {
        Rule::def_function => Statement::DefFunction(def_function(p)),
        Rule::effect => Statement::Effect(effect(p)),
        Rule::let_variable => Statement::LetVariable(let_variable(p)),
        _ => unreachable!(),
    }
}

fn effect(p: Pair<Rule>) -> Effect {
    let b = &p.as_str()[0..2] == "..";
    Effect::new(expression(p.into_inner().next().unwrap()), b)
}

fn expression(p: Pair<Rule>) -> Expression {
    let p = p.into_inner().next().unwrap();

    match p.as_rule() {
        Rule::boolean => Expression::Boolean(FromStr::from_str(p.as_str()).unwrap()),
        Rule::dictionary => dictionary(p),
        Rule::list => list(p),
        Rule::nil => Expression::Nil,
        Rule::number => Expression::Number(FromStr::from_str(p.as_str()).unwrap()),
        Rule::string => Expression::String({
            let s = p.as_str();

            s[1..(s.len() - 1)]
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .into()
        }),
        Rule::name => Expression::Name(p.as_str().into()),
        Rule::application => application(p),
        _ => unreachable!(),
    }
}

fn dictionary(p: Pair<Rule>) -> Expression {
    Expression::Dictionary(p.into_inner().map(dictionary_element).collect())
}

fn dictionary_element(p: Pair<Rule>) -> Expansion<(Expression, Expression)> {
    match p.as_rule() {
        Rule::key_value_pair => {
            let mut i = p.into_inner();
            Expansion::Unexpanded((expression(i.next().unwrap()), expression(i.next().unwrap())))
        }
        Rule::expanded_expression => {
            Expansion::Expanded(expression(p.into_inner().next().unwrap()))
        }
        _ => unreachable!(),
    }
}

fn list(p: Pair<Rule>) -> Expression {
    Expression::List(p.into_inner().map(list_element).collect())
}

fn list_element(p: Pair<Rule>) -> Expansion<Expression> {
    match p.as_rule() {
        Rule::expression => Expansion::Unexpanded(expression(p)),
        Rule::expanded_expression => {
            Expansion::Expanded(expression(p.into_inner().next().unwrap()))
        }
        _ => unreachable!(),
    }
}

fn application(p: Pair<Rule>) -> Expression {
    let mut i = p.into_inner();

    Expression::App(
        Box::new(expression(i.next().unwrap())),
        arguments(i.next().unwrap()),
    )
}

fn arguments(p: Pair<Rule>) -> Arguments {
    let mut ps = vec![];
    let mut ks = vec![];

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::positional_arguments => ps = positional_arguments(p),
            Rule::keyword_arguments => ks = keyword_arguments(p),
            _ => unreachable!(),
        }
    }

    Arguments::new(ps, ks)
}

fn positional_arguments(p: Pair<Rule>) -> Vec<Expansion<Expression>> {
    p.into_inner()
        .map(|p| match p.as_rule() {
            Rule::expression => Expansion::Unexpanded(expression(p)),
            Rule::expanded_argument => {
                Expansion::Expanded(expression(p.into_inner().next().unwrap()))
            }

            _ => unreachable!(),
        })
        .collect()
}

fn keyword_arguments(p: Pair<Rule>) -> Vec<Expansion<KeywordArgument>> {
    p.into_inner().map(keyword_argument).collect()
}

fn keyword_argument(p: Pair<Rule>) -> Expansion<KeywordArgument> {
    match p.as_rule() {
        Rule::keyword_argument => {
            let mut i = p.into_inner();

            Expansion::Unexpanded(KeywordArgument::new(
                i.next().unwrap().as_str().into(),
                expression(i.next().unwrap()),
            ))
        }
        Rule::expanded_argument => Expansion::Expanded(expression(p.into_inner().next().unwrap())),
        _ => unreachable!(),
    }
}

fn signature(p: Pair<Rule>) -> Signature {
    let mut i = p.into_inner();

    Signature::new(
        half_signature(i.next().unwrap()),
        i.next()
            .map(half_signature)
            .unwrap_or(HalfSignature::default()),
    )
}

fn half_signature(p: Pair<Rule>) -> HalfSignature {
    let mut rs = vec![];
    let mut os = vec![];
    let mut r = "".into();

    for p in p.into_inner() {
        match p.as_rule() {
            Rule::name => rs.push(p.as_str().into()),
            Rule::optional_parameter => os.push(optional_parameter(p)),
            Rule::rest_parameter => r = p.into_inner().next().unwrap().as_str().into(),
            _ => unreachable!(),
        }
    }

    HalfSignature::new(rs, os, r)
}

fn optional_parameter(p: Pair<Rule>) -> OptionalParameter {
    let mut i = p.into_inner();

    OptionalParameter::new(
        i.next().unwrap().as_str().into(),
        expression(i.next().unwrap()),
    )
}

fn def_function(p: Pair<Rule>) -> DefFunction {
    let mut i = p.into_inner();

    let n = i.next().unwrap().as_str().into();
    let s = signature(i.next().unwrap());

    let mut ss = vec![];
    let mut b = Expression::Nil;

    for p in i {
        match p.as_rule() {
            Rule::inner_statement => ss.push(inner_statement(p)),
            Rule::expression => b = expression(p),
            _ => unreachable!(),
        }
    }

    DefFunction::new(n, s, ss, b)
}

fn inner_statement(p: Pair<Rule>) -> InnerStatement {
    let p = p.into_inner().next().unwrap();

    match p.as_rule() {
        Rule::def_function => InnerStatement::DefFunction(def_function(p)),
        Rule::let_variable => InnerStatement::LetVariable(let_variable(p)),
        _ => unreachable!(),
    }
}

fn let_variable(p: Pair<Rule>) -> LetVariable {
    let mut i = p.into_inner();

    LetVariable::new(
        i.next().unwrap().as_str().into(),
        expression(i.next().unwrap()),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    const EXPRESSIONS: &[&'static str] = &["nil", "123", "0.1", "-123", "-0.1", "true", "false"];

    #[test]
    fn name() {
        for s in &[
            "x",
            "x1",
            "func",
            "PureFunc",
            "alskfj1",
            "?",
            "is_boolean?",
            "isBoolean?",
        ] {
            println!("{}", s);
            LanguageParser::parse(Rule::name, s).unwrap();
        }
    }

    #[test]
    fn boolean() {
        for s in vec!["true", "false"] {
            LanguageParser::parse(Rule::boolean, s).unwrap();
        }
    }

    #[test]
    fn dictionary_parser() {
        for (s, e) in vec![
            ("{}", Expression::Dictionary(vec![])),
            (
                "{\"foo\" 42 ..dict}",
                Expression::Dictionary(vec![
                    Expansion::Unexpanded((
                        Expression::String("foo".into()),
                        Expression::Number(42.0),
                    )),
                    Expansion::Expanded(Expression::Name("dict".into())),
                ]),
            ),
        ] {
            assert_eq!(
                expression(
                    LanguageParser::parse(Rule::expression, s)
                        .unwrap()
                        .next()
                        .unwrap()
                ),
                e
            );
        }
    }

    #[test]
    fn list_parser() {
        for (s, e) in vec![
            ("[]", Expression::List(vec![])),
            (
                "[\"foo\" 42 ..list]",
                Expression::List(vec![
                    Expansion::Unexpanded(Expression::String("foo".into())),
                    Expansion::Unexpanded(Expression::Number(42.0)),
                    Expansion::Expanded(Expression::Name("list".into())),
                ]),
            ),
        ] {
            assert_eq!(
                expression(
                    LanguageParser::parse(Rule::expression, s)
                        .unwrap()
                        .next()
                        .unwrap()
                ),
                e
            );
        }
    }

    #[test]
    fn nil() {
        LanguageParser::parse(Rule::nil, "nil").unwrap();
    }

    #[test]
    fn number() {
        for s in vec!["123", "-0.1"] {
            LanguageParser::parse(Rule::number, s).unwrap();
        }
    }

    #[test]
    fn string() {
        for s in vec![
            "\"\"",
            "\"a\"",
            "\"abc\"",
            "\"Hi!\"",
            "\"\\\"\"",
            "\"\\\\\"",
            "\"\\n\"",
            "\"\\r\"",
            "\"\\t\"",
            "\"\\\"\\\\\\n\\r\\t\"",
        ] {
            LanguageParser::parse(Rule::string, s).unwrap();
        }
    }

    #[test]
    fn escaped_string() {
        assert_eq!(
            expression(
                LanguageParser::parse(Rule::expression, "\"\\\"\\\\\\n\\r\\t\"")
                    .unwrap()
                    .next()
                    .unwrap()
            ),
            Expression::String("\"\\\n\r\t".into()),
        );
    }

    #[test]
    fn anonymous_function() {
        for s in &[
            "(\\ () 123)",
            "(\\ (x) x)",
            "(\\ (x y) (+ x y))",
            "(\\ (x y . ..options) \"Hi!\")",
        ] {
            println!("{}", s);
            LanguageParser::parse(Rule::anonymous_function, s).unwrap();
        }
    }

    #[test]
    fn match_expression() {
        for s in &["(match a _ 42)", "(match (f x y) \"foo\" \"bar\" 42 nil)"] {
            println!("{}", s);
            LanguageParser::parse(Rule::match_expression, s).unwrap();
        }
    }

    #[test]
    fn application_tokenizer() {
        for s in &[
            "(foo)",
            "(f x)",
            "(f x y)",
            "(f ..args)",
            "(f x ..args)",
            "(f . x 123)",
            "(f . x 123 y 456)",
            "(f . ..kwargs)",
            "(f . x 123 ..kwargs)",
        ] {
            println!("{}", s);
            LanguageParser::parse(Rule::application, s).unwrap();
        }
    }

    #[test]
    fn application_parser() {
        for (s, e) in vec![
            (
                "(f)",
                Expression::App(
                    Box::new(Expression::Name("f".into())),
                    Arguments::new(vec![], vec![]),
                ),
            ),
            (
                "(f . x 42 ..options)",
                Expression::App(
                    Box::new(Expression::Name("f".into())),
                    Arguments::new(
                        vec![],
                        vec![
                            Expansion::Unexpanded(KeywordArgument::new(
                                "x".into(),
                                Expression::Number(42.0),
                            )),
                            Expansion::Expanded(Expression::Name("options".into())),
                        ],
                    ),
                ),
            ),
        ] {
            assert_eq!(
                expression(
                    LanguageParser::parse(Rule::expression, s)
                        .unwrap()
                        .next()
                        .unwrap()
                ),
                e
            );
        }
    }

    #[test]
    fn expression_tokenizer() {
        for s in EXPRESSIONS {
            println!("{}", s);
            LanguageParser::parse(Rule::expression, s).unwrap();
        }
    }

    #[test]
    fn signature_parser() {
        for (s, x) in vec![
            ("", Signature::default()),
            (
                "x y",
                Signature::new(
                    HalfSignature::new(vec!["x".into(), "y".into()], vec![], "".into()),
                    HalfSignature::default(),
                ),
            ),
            (
                "(x 42)",
                Signature::new(
                    HalfSignature::new(
                        vec![],
                        vec![OptionalParameter::new("x".into(), Expression::Number(42.0))],
                        "".into(),
                    ),
                    HalfSignature::default(),
                ),
            ),
            (
                ". x y",
                Signature::new(
                    HalfSignature::default(),
                    HalfSignature::new(vec!["x".into(), "y".into()], vec![], "".into()),
                ),
            ),
            (
                "..rest",
                Signature::new(
                    HalfSignature::new(vec![], vec![], "rest".into()),
                    HalfSignature::default(),
                ),
            ),
        ] {
            assert_eq!(
                signature(
                    LanguageParser::parse(Rule::signature, s)
                        .unwrap()
                        .next()
                        .unwrap()
                ),
                x
            );
        }
    }

    #[test]
    fn def_function_tokenizer() {
        for s in &[
            "(def (func) 123)",
            "(  def \n(func) (let  x  42\t) x)",
            "(def (func x) x)",
            "(def (func x y) x)",
            "(def (func (x 123)) x)",
            "(def (func ..args) x)",
            "(def (func . x) x)",
            "(def (func . x y) x)",
            "(def (func . (x 123)) x)",
            "(def (func . ..kwargs) x)",
            "(def (func x y . ..kwargs) x)",
        ] {
            println!("{}", s);
            LanguageParser::parse(Rule::def_function, s).unwrap();
        }
    }

    #[test]
    fn def_function_parser() {
        for (s, f) in vec![
            (
                "(def (f x) x)",
                DefFunction::new(
                    "f".into(),
                    Signature::new(
                        HalfSignature::new(vec!["x".into()], vec![], "".into()),
                        HalfSignature::default(),
                    ),
                    vec![],
                    Expression::Name("x".into()),
                ),
            ),
            (
                "(def (f x) (let y 42) x)",
                DefFunction::new(
                    "f".into(),
                    Signature::new(
                        HalfSignature::new(vec!["x".into()], vec![], "".into()),
                        HalfSignature::default(),
                    ),
                    vec![InnerStatement::LetVariable(LetVariable::new(
                        "y".into(),
                        Expression::Number(42.into()),
                    ))],
                    Expression::Name("x".into()),
                ),
            ),
            (
                "(def (f x) (def (g y) y) x)",
                DefFunction::new(
                    "f".into(),
                    Signature::new(
                        HalfSignature::new(vec!["x".into()], vec![], "".into()),
                        HalfSignature::default(),
                    ),
                    vec![InnerStatement::DefFunction(DefFunction::new(
                        "g".into(),
                        Signature::new(
                            HalfSignature::new(vec!["y".into()], vec![], "".into()),
                            HalfSignature::default(),
                        ),
                        vec![],
                        Expression::Name("y".into()),
                    ))],
                    Expression::Name("x".into()),
                ),
            ),
            (
                "(def (f x) (def (g y) (let z 42) y) x)",
                DefFunction::new(
                    "f".into(),
                    Signature::new(
                        HalfSignature::new(vec!["x".into()], vec![], "".into()),
                        HalfSignature::default(),
                    ),
                    vec![InnerStatement::DefFunction(DefFunction::new(
                        "g".into(),
                        Signature::new(
                            HalfSignature::new(vec!["y".into()], vec![], "".into()),
                            HalfSignature::default(),
                        ),
                        vec![InnerStatement::LetVariable(LetVariable::new(
                            "z".into(),
                            Expression::Number(42.0),
                        ))],
                        Expression::Name("y".into()),
                    ))],
                    Expression::Name("x".into()),
                ),
            ),
        ] {
            assert_eq!(
                def_function(
                    LanguageParser::parse(Rule::def_function, s)
                        .unwrap()
                        .next()
                        .unwrap()
                ),
                f
            );
        }
    }

    #[test]
    fn let_variable() {
        for s in &["(let x 123)", "(   let   thisIsNumber \t\n 123\n)"] {
            println!("{}", s);
            LanguageParser::parse(Rule::let_variable, s).unwrap();
        }
    }

    #[test]
    fn import() {
        for s in &["(import \"foo\")", "(import \"x\")"] {
            LanguageParser::parse(Rule::import, s).unwrap();
        }
    }

    #[test]
    fn effect_parser() {
        for (s, x) in vec![
            ("nil", Effect::new(Expression::Nil, false)),
            ("..nil", Effect::new(Expression::Nil, true)),
        ] {
            assert_eq!(
                effect(
                    LanguageParser::parse(Rule::effect, s)
                        .unwrap()
                        .next()
                        .unwrap()
                ),
                x
            );
        }
    }

    #[test]
    fn main_module_tokenizer() {
        for s in &[
            "",
            " 123 nil \n \ttrue",
            "; comment",
            "; comment\n123",
            "(def (f) 123)",
            "(let x 123)",
        ] {
            println!("{}", s);
            LanguageParser::parse(Rule::main_module, s).unwrap();
        }
    }

    #[test]
    fn main_module_parser() {
        for (s, m) in vec![
            ("", Module::new(vec![], vec![])),
            (
                "123",
                Module::new(
                    vec![],
                    vec![Statement::Effect(Effect::new(
                        Expression::Number(123.0),
                        false,
                    ))],
                ),
            ),
            (
                "true nil 123 \"foo\"",
                Module::new(
                    vec![],
                    vec![
                        Statement::Effect(Effect::new(Expression::Boolean(true), false)),
                        Statement::Effect(Effect::new(Expression::Nil, false)),
                        Statement::Effect(Effect::new(Expression::Number(123.0), false)),
                        Statement::Effect(Effect::new(Expression::String("foo".into()), false)),
                    ],
                ),
            ),
            (
                " 123 ; foo \n456",
                Module::new(
                    vec![],
                    vec![
                        Statement::Effect(Effect::new(Expression::Number(123.0), false)),
                        Statement::Effect(Effect::new(Expression::Number(456.0), false)),
                    ],
                ),
            ),
            (
                "(let name 42)",
                Module::new(
                    vec![],
                    vec![Statement::LetVariable(LetVariable::new(
                        "name".into(),
                        Expression::Number(42.0),
                    ))],
                ),
            ),
            (
                "(def (f) 42)",
                Module::new(
                    vec![],
                    vec![Statement::DefFunction(DefFunction::new(
                        "f".into(),
                        Signature::default(),
                        vec![],
                        Expression::Number(42.0),
                    ))],
                ),
            ),
            (
                "(import \"http\")",
                Module::new(vec![Import::new("http".into())], vec![]),
            ),
        ] {
            println!("{:?}", s);
            println!("{:?}", m);
            assert_eq!(main_module(s), Ok(m.clone()));
        }
    }

    #[test]
    fn sub_module_parser() {
        for (s, m) in vec![
            ("", Module::new(vec![], vec![])),
            (
                "(let name 42)",
                Module::new(
                    vec![],
                    vec![Statement::LetVariable(LetVariable::new(
                        "name".into(),
                        Expression::Number(42.0),
                    ))],
                ),
            ),
            (
                "(def (f) 42)",
                Module::new(
                    vec![],
                    vec![Statement::DefFunction(DefFunction::new(
                        "f".into(),
                        Signature::default(),
                        vec![],
                        Expression::Number(42.0),
                    ))],
                ),
            ),
            (
                "(import \"http\")",
                Module::new(vec![Import::new("http".into())], vec![]),
            ),
        ] {
            println!("{:?}", s);
            println!("{:?}", m);
            assert_eq!(sub_module(s), Ok(m.clone()));
        }
    }

    #[test]
    fn sub_module_parser_error() {
        assert!(sub_module("(write 42)").is_err());
    }
}
