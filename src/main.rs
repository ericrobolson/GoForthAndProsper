use std::io;
use std::io::Write;

mod context;
mod dictionary;
mod id;
mod stack;

fn main() {
    let mut forth = context::Context::new(i16::MAX as usize, 666);

    loop {
        print!("go-forth> ");
        io::stdout().flush().unwrap();

        // Do the reading
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(size) => match forth.eval(input) {
                Ok(result) => match result {
                    context::Return::Ok => {
                        println!("OK -> STACK {:?}", forth.stack());
                    }
                    context::Return::Shutdown => {
                        println!("OK: Shutting down...");
                        return;
                    }
                    context::Return::Yielding => {
                        println!("Doing a yield.");
                    }
                },
                Err(error) => {
                    println!("ERROR: {:?}", error);
                }
            },
            Err(error) => {
                println!("ERROR: {:?}", error);
            }
        }

        // Do the loop
    }
}
