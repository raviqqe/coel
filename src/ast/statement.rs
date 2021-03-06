use super::def_function::DefFunction;
use super::effect::Effect;
use super::let_variable::LetVariable;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Effect(Effect),
    DefFunction(DefFunction),
    LetVariable(LetVariable),
}
