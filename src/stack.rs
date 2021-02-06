/// A list of errors a stack operation may return.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StackErr {
    Overflow,
    Underflow,
}

/// Stack data structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack<Data> {
    data: Vec<Data>,
    capacity: usize,
}

impl<Data> Stack<Data> {
    /// Creates a new stack with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn data(&self) -> &[Data] {
        &self.data
    }

    /// Pushes a new item onto the stack.
    pub fn push(&mut self, data: Data) -> Result<(), StackErr> {
        if self.data.len() < self.capacity {
            self.data.push(data);

            Ok(())
        } else {
            Err(StackErr::Overflow)
        }
    }

    /// Clears all values from the stack.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Pops an item off the stack.
    pub fn pop(&mut self) -> Result<Data, StackErr> {
        match self.data.pop() {
            Some(d) => Ok(d),
            None => Err(StackErr::Underflow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_capacity() {
        let cap = 30201;
        let stack = Stack::<i32>::new(cap);
        assert_eq!(cap, stack.capacity);
        assert_eq!(true, stack.data.is_empty());
    }

    #[test]
    fn new_push_pushes_item() {
        let cap = 30201;
        let mut stack = Stack::new(cap);
        let i = 392;
        let result = stack.push(i);
        assert_eq!(true, result.is_ok());
        assert_eq!(i, stack.data[0]);

        let i = 420;
        let result = stack.push(i);
        assert_eq!(true, result.is_ok());
        assert_eq!(i, stack.data[1]);
    }

    #[test]
    fn new_push_would_overflow_returns_err() {
        let cap = 1;
        let mut stack = Stack::new(cap);
        let i = 392;
        let result = stack.push(i);
        assert_eq!(true, result.is_ok());
        assert_eq!(i, stack.data[0]);

        let i = 420;
        let result = stack.push(i);
        assert_eq!(false, result.is_ok());
        assert_eq!(StackErr::Overflow, result.unwrap_err());
    }

    #[test]
    fn new_pop_returns_item() {
        let cap = 30201;
        let mut stack = Stack::new(cap);
        let i = 392;
        let j = 420;

        stack.push(i).unwrap();
        stack.push(j).unwrap();

        let result = stack.pop();
        assert_eq!(true, result.is_ok());
        assert_eq!(j, result.unwrap());

        let result = stack.pop();
        assert_eq!(true, result.is_ok());
        assert_eq!(i, result.unwrap());
    }

    #[test]
    fn new_pop_underflow_returns_err() {
        let cap = 30201;
        let mut stack = Stack::new(cap);

        stack.push(46).unwrap();
        stack.push(46).unwrap();

        let result = stack.pop();
        let result = stack.pop();

        let result = stack.pop();
        assert_eq!(false, result.is_ok());
        assert_eq!(StackErr::Underflow, result.unwrap_err());
    }
}
