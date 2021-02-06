use crate::{dictionary, id::Id, stack};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Interpreting,
    Compiling,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Return {
    Ok,
    Yielding,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextErr {
    StackErr(stack::StackErr),
    DivideByZero,
    Parse(std::num::ParseIntError),
    DictionaryErr(dictionary::DictionaryErr),
    AccessedUndefinedAtAddr(usize),
}

impl From<stack::StackErr> for ContextErr {
    fn from(se: stack::StackErr) -> Self {
        Self::StackErr(se)
    }
}

impl From<std::num::ParseIntError> for ContextErr {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::Parse(e)
    }
}

impl From<dictionary::DictionaryErr> for ContextErr {
    fn from(de: dictionary::DictionaryErr) -> Self {
        Self::DictionaryErr(de)
    }
}

pub type Procedure = Box<dyn Fn(&mut Context) -> Result<(), ContextErr>>;

macro_rules! builtin_word {
    ($context:ident : $word:expr => $execution:expr) => {
        let action: Procedure = { Box::new($execution) };

        $context
            .dictionary
            .insert(Some($word.into()), Rc::new(Word::Builtin(action)))?;
    };
}

pub enum Word {
    Builtin(Procedure),
    /// A custom, user defined word. If multiple words are chained together to make up this word, they are stored in the body and pushed to the call stack. The size of 13 is arbitrary, and open to change.
    Custom {
        body: [Rc<Word>; 13],
    },
    Data(Datum),
}

impl std::fmt::Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Word::Builtin(_) => f.write_str(&format!("Builtin, can't deal")),
            Word::Custom { body } => f.write_str(&format!("Custom {:?}", body)),
            Word::Data(d) => f.write_str(&format!("Data: {:?}", d)),
        }
    }
}

/// The basic types that may be put on the stack
pub type Datum = i32;

pub struct Context {
    stack: stack::Stack<Datum>,
    mode: Mode,
    dictionary: dictionary::Dictionary<Id, Rc<Word>>,
    fsm: Fsm,
}

enum Fsm {
    Execute,
    GetVariable,
}

impl Context {
    /// Creates a new context for interpreting.
    pub fn new(stack_capacity: usize, dictionary_capacity: usize) -> Self {
        let mut forth = Self {
            fsm: Fsm::Execute,
            stack: stack::Stack::new(stack_capacity),
            mode: Mode::Interpreting,
            dictionary: dictionary::Dictionary::new(dictionary_capacity),
        };

        forth.reset();

        forth
    }

    /// Resets the context to a pristine state.
    pub fn reset(&mut self) {
        self.fsm = Fsm::Execute;
        self.dictionary.clear();
        self.stack.clear();
        self.mode = Mode::Interpreting;
        self.set_primitives().unwrap();
    }

    /// Pushes a new value onto the stack.
    pub fn push(&mut self, data: Datum) -> Result<(), stack::StackErr> {
        self.stack.push(data)
    }

    /// Pops a value off the stack.
    pub fn pop(&mut self) -> Result<Datum, stack::StackErr> {
        self.stack.pop()
    }

    /// Returns a read-only handle to the stack.
    pub fn stack(&self) -> &[Datum] {
        &self.stack.data()
    }

    /// Returns a read-only handle to the dictionary.
    pub fn dictionary(&self) -> &[(Option<Id>, Rc<Word>)] {
        self.dictionary.dictionary()
    }

    /// Evaluates a line of code. By default, tokens are separated by whitespace.
    pub fn eval(&mut self, line: String) -> Result<Return, ContextErr> {
        // a) Skip leading spaces and parse a name (see 3.4.1);
        for word_str in line.split_whitespace() {
            match self.fsm {
                Fsm::Execute => {
                    match word_str {
                        "bye" => {
                            return Ok(Return::Shutdown);
                        }
                        "yield" => {
                            todo!("There's a bug where yielding doesn't resume. It just chops off other stuff.");
                            return Ok(Return::Yielding);
                        }
                        "var" => {
                            // https://forth-standard.org/standard/core/VARIABLE
                            // Idea: if you ever need to extend this, consider a FSM to wait for another input
                            self.fsm = Fsm::GetVariable;
                        }
                        _ => {
                            // b) Search the dictionary name space (see 3.4.2).
                            let word = match self.find_word(word_str) {
                                Some(word) => word,
                                None => {
                                    let i = self.convert_to_number(word_str)?;

                                    Rc::new(Word::Data(i))
                                }
                            };

                            match self.mode {
                                Mode::Interpreting => {
                                    self.run_word(word)?;
                                }
                                Mode::Compiling => {
                                    todo!("Compiling");
                                }
                            }
                        }
                    }
                }
                Fsm::GetVariable => {
                    // add a value to the dict without a key.
                    let addr = self
                        .dictionary
                        .insert(None, Rc::new(Word::Data(Datum::default())))?;

                    self.dictionary
                        .insert(Some(word_str.into()), Rc::new(Word::Data(addr as Datum)))?;

                    // Switch back to execution mode
                    self.fsm = Fsm::Execute;
                }
            }
        }

        Ok(Return::Ok)
    }

    fn run_word(&mut self, word: Rc<Word>) -> Result<(), ContextErr> {
        match *word {
            Word::Builtin(ref built_in) => {
                built_in(self)?;
            }
            Word::Data(ref lit) => {
                self.stack.push(*lit)?;
            }
            Word::Custom { ref body } => {
                // Execute all queued methods
                for call in body.iter() {
                    match **call {
                        _ => {
                            self.run_word(call.clone())?;
                        }
                    }
                }

                todo!()
            }
        }

        Ok(())
    }

    fn find_word(&self, word: &str) -> Option<Rc<Word>> {
        match self.dictionary.get(word.into()) {
            Some(word) => Some(word.clone()),
            None => None,
        }
    }

    fn convert_to_number(&self, word: &str) -> Result<Datum, ContextErr> {
        Ok(word.parse::<Datum>()?)
    }

    fn set_primitives(&mut self) -> Result<(), ContextErr> {
        builtin_word!(self : "does>" => |context| {
            todo!();
        });

        builtin_word!(self : "create" => |context| {
            todo!();
        });

        builtin_word!(self : "drop" => |context| {
            context.stack.pop()?;
            Ok(())
        });

        builtin_word!(self : "print" => |context| {
            // Print a value
            let val = context.stack.pop()?;
            println!(":: {:?}", val);
            context.stack.push(val)?;
            Ok(())
        });

        builtin_word!(self : "!" => |context| {
            let addr = context.stack.pop()?;
            let x = context.stack.pop()?;

            context.dictionary.set_from_addr(addr as usize, Rc::new(Word::Data(x)))?;

            Ok(())
        });

        builtin_word!(self : "dict" => |context| {
            for (i, kv) in context.dictionary.dictionary().iter().enumerate(){
                println!("{:?}: DICT: {:?}",i, kv);
            }
            Ok(())
        });

        builtin_word!(self : "@" => |context| {
            // TODO: test
            // https://forth-standard.org/standard/core/Fetch
            let a_addr = context.stack.pop()?;
            let a_addr = a_addr as usize;
            match  context.dictionary.get_from_addr(a_addr) {
                Some((key, value)) => {
                    match **value {
                        Word::Data(i) => {
                            context.stack.push(i)?;
                        },
                        _ => {
                            let mut found = false;

                            if let Some(key) = key {
                                if let Some(addr) = context.dictionary.get_addr(*key){
                                    found = true;
                                    context.stack.push(addr as Datum)?;
                                }
                            }

                            if !found{
                                todo!("Attempted to get key that didn't exist.");
                            }
                        }
                    }

                },
                None => {
                    return Err(ContextErr::AccessedUndefinedAtAddr(a_addr));
                }
            }

            Ok(())
        });

        builtin_word!(self : "-" => |context| {
            let n1 = context.stack.pop()?;
            let n2 = context.stack.pop()?;

            context.stack.push(n1 - n2)?;

            Ok(())
        });

        builtin_word!(self : "+" => |context| {
            let n1 = context.stack.pop()?;
            let n2 = context.stack.pop()?;
            context.stack.push(n1 + n2)?;
            Ok(())
        });

        builtin_word!(self : "*" => |context| {
            let n1 = context.stack.pop()?;
            let n2 = context.stack.pop()?;
            context.stack.push(n1 * n2)?;
            Ok(())
        });

        builtin_word!(self : "/" => |context| {
            let n1 = context.stack.pop()?;
            let n2 = context.stack.pop()?;
            if n2 == 0 {
                return Err(ContextErr::DivideByZero);
            }

            context.stack.push(n1 / n2)?;
            Ok(())
        });

        builtin_word!(self : "dup" => |context |{
            let n = context.stack.pop()?;
            context.stack.push(n)?;
            context.stack.push(n)?;
            Ok(())
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_divides() {
        let mut f = Context::new(333, 343);
        f.eval("4 7 /".into()).unwrap();
        assert_eq!(1, f.stack()[0]);

        f.reset();

        f.eval("3 -9 /".into()).unwrap();
        assert_eq!(-3, f.stack()[0]);

        f.reset();

        assert_eq!(
            ContextErr::DivideByZero,
            f.eval("0 -9 /".into()).unwrap_err()
        );
    }

    #[test]
    fn test_mul_multiplies() {
        let mut f = Context::new(333, 343);
        f.eval("4 7 *".into()).unwrap();
        assert_eq!(28, f.stack()[0]);

        f.eval("-9 *".into()).unwrap();
        assert_eq!(-252, f.stack()[0]);
    }

    #[test]
    fn test_sub_subtracts() {
        let mut f = Context::new(333, 343);
        f.eval("1 2 -".into()).unwrap();
        assert_eq!(1, f.stack()[0]);

        f.eval("-9 -".into()).unwrap();
        assert_eq!(-10, f.stack()[0]);
    }

    #[test]
    fn test_plus_adds() {
        let mut f = Context::new(333, 343);
        f.eval("1 2 +".into()).unwrap();
        assert_eq!(3, f.stack()[0]);

        f.eval("1 +".into()).unwrap();
        assert_eq!(4, f.stack()[0]);
    }

    #[test]
    fn test_DUP_duplicates_top_of_stack() {
        let mut f = Context::new(333, 343);
        f.eval("1 DUP".into()).unwrap();
        assert_eq!(1, f.stack()[0]);
        assert_eq!(1, f.stack()[1]);
    }

    #[test]
    fn test_bye_returns_exist() {
        assert_eq!(true, false);
    }

    #[test]
    fn variable() {
        let mut f = Context::new(333, 343);

        f.eval("variable balance 123 balance ! balance @".into())
            .unwrap();

        assert_eq!(f.stack()[0], 123);
    }
}
