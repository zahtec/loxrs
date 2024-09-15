use crate::{expressions::Literal, interpreter::Interpreter};
use std::rc::Rc;

// pub trait DebugFn: Fn(&mut Interpreter, &Vec<String>, Vec<Literal>)

// impl<F> DebugFn for F where F: Fn(&mut Interpreter, &Vec<String>, Vec<Literal>) {}

// impl std::fmt::Debug for DisplayFnT {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "DisplayFunction")
//     }
// }

#[derive(Clone, Debug)]
pub struct Callable {
    parameters: Vec<String>,
    func: Rc<dyn Fn(&mut Interpreter, &Vec<String>, Vec<Literal>) -> Result<Literal, ()>>,
}

impl Callable {
    pub fn new(
        parameters: Vec<String>,
        func: Rc<dyn Fn(&mut Interpreter, &Vec<String>, Vec<Literal>) -> Result<Literal, ()>>,
    ) -> Callable {
        Callable { parameters, func }
    }

    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, ()> {
        (self.func)(interpreter, &self.parameters, arguments)
    }
}
