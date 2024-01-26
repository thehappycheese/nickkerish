use uiua::{
    Uiua
};
use crate::util::TryToJsonBytesString;
use anyhow::Result;

pub fn run_wrapped(code:&str) -> String{
    let mut runtime = Uiua::with_native_sys();
    match runtime.compile_run(|compiler| compiler.load_str(&code)) {
        Ok(_compiler) => {
            let stack = runtime.get_stack();
            let strings:Vec<String> = stack.iter().map(|value| value.show()).collect();
            
            println!("Got strings of length {}",strings.len());
            strings.join("\n")

        }
        Err(e) => {
            e.message()
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_run_wrapped() {
        let code = "+ 1 1";
        let result = run_wrapped(code);
        assert_eq!(result, "2");
    }

    #[test]
    fn test_error() {
        let code = "1 + 1";
        let result = run_wrapped(code);
        println!("{}",result);
    }
}