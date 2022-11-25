// Take a javascript file and convert it into an equivalent javascript file
// composed only of the characters ({[/>+!-=\]})

// This takes advantage of Javascript's Type Coersion
// "a" + 0 = "a0"
// true + 0 = 1
// 2+"2" = "22"
// 2-"2" = 0
// [] is approx equal to ""
// +[] = +"" = 0

use clap::Parser;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;

static ZERO: &str = "+[]"; // Type gets coerced to zero.
static ONE: &str = "+!![]"; // Type gets coerced to true, then to 1

fn number(n: usize) -> String {
    if n == 0 {
        return String::from(ZERO);
    }
    // Create a vector of size n, filled with ones
    let vec = vec![ONE; n];
    // Joins the ones to sum to the target num
    vec.join("")
}

fn from_string(map: &HashMap<char, String>, s: &str) -> String {
    let mut vec: Vec<String> = Vec::new();
    for c in s.chars() {
        let char_str = map.get(&c);
        if let Some(char_str) = char_str {
            vec.push(char_str.clone());
        } else {
            vec.push(format!(
                "([]+[])[{}][{}]({})",
                from_string(&map, "constructor"),
                from_string(&map, "fromCharCode"),
                number(c as usize)
            ));
        }
    }
    vec.join("+")
}

fn compile(map: &HashMap<char, String>, code: &str) -> String {
    format!(
        "(()=>{{}})[{}]({})()",
        from_string(&map, "constructor"),
        from_string(&map, code)
    )
}

struct Formatter(String, HashMap<char, String>);

impl Formatter {
    fn index_by_num(&self, index: usize) -> String {
        format!("({})[{}]", self.0, number(index))
    }
    fn index_by_str(&self, index: &'static str) -> String {
        format!("({})[{}]", self.0, from_string(&self.1, index))
    }
    fn index(&self, index: JavascriptIndex) -> String {
        match index {
            JavascriptIndex::NUMBER(num) => self.index_by_num(num),
            JavascriptIndex::STRING(s) => self.index_by_str(s),
        }
    }
}

enum JavascriptIndex {
    NUMBER(usize),
    STRING(&'static str),
}

fn build_char_map() -> HashMap<char, String> {
    let mut map: HashMap<char, String> = HashMap::new();

    // +{} is NaN, adding [] makes this a string
    map.insert('a', String::from("(+{}+[])[+!![]]"));
    // ({}+[]) yields '[object Object]'
    let phrase = Formatter("{}+[]".to_string(), map.clone());
    map.insert('o', phrase.index(JavascriptIndex::NUMBER(1)).to_string());
    map.insert('b', phrase.index(JavascriptIndex::NUMBER(2)).to_string());
    map.insert('j', phrase.index(JavascriptIndex::NUMBER(3)).to_string());
    map.insert('e', phrase.index(JavascriptIndex::NUMBER(4)).to_string());
    map.insert('c', phrase.index(JavascriptIndex::NUMBER(5)).to_string());
    map.insert('t', phrase.index(JavascriptIndex::NUMBER(6)).to_string());
    map.insert(' ', phrase.index(JavascriptIndex::NUMBER(7)).to_string());
    // ![]+[] yields the string 'false'
    let phrase = Formatter("![]+[]".to_string(), map.clone());
    map.insert('f', phrase.index(JavascriptIndex::NUMBER(0)).to_string());
    map.insert('l', phrase.index(JavascriptIndex::NUMBER(2)).to_string());
    map.insert('s', phrase.index(JavascriptIndex::NUMBER(3)).to_string());
    // !![]+[] yields the string 'true'
    let phrase = Formatter("!![]+[]".to_string(), map.clone());
    map.insert('t', phrase.index(JavascriptIndex::NUMBER(0)).to_string());
    map.insert('r', phrase.index(JavascriptIndex::NUMBER(1)).to_string());
    map.insert('u', phrase.index(JavascriptIndex::NUMBER(2)).to_string());
    // (one/zero)+[] yields the string 'Infinity'
    let phrase = Formatter("+!![]/+![]+[]".to_string(), map.clone());
    map.insert('n', phrase.index(JavascriptIndex::NUMBER(4)).to_string());
    map.insert('i', phrase.index(JavascriptIndex::NUMBER(3)).to_string());
    // (1)["toString"] yields the string 'function toString() { [native code] }'
    let phrase =
        Formatter("[]+[]".to_string(), map.clone()).index(JavascriptIndex::STRING("constructor"));
    let phrase = Formatter(format!("[]+{}", phrase), map.clone());
    map.insert('S', phrase.index(JavascriptIndex::NUMBER(9)).to_string());
    map.insert('g', phrase.index(JavascriptIndex::NUMBER(14)).to_string());
    // get the constructor of the regex
    let phrase =
        Formatter("/-/".to_string(), map.clone()).index(JavascriptIndex::STRING("constructor"));
    let phrase = Formatter(format!("[]+{}", phrase), map.clone());
    map.insert('p', phrase.index(JavascriptIndex::NUMBER(14)).to_string());

    let phrase = Formatter("/\\\\/+[]".to_string(), map.clone());
    map.insert('\\', phrase.index(JavascriptIndex::NUMBER(1)).to_string());

    map.insert(
        'd',
        format!(
            "({})[{}]({})",
            number(13),
            from_string(&map, "toString"),
            number(14)
        ),
    );
    map.insert(
        'h',
        format!(
            "({})[{}]({})",
            number(17),
            from_string(&map, "toString"),
            number(18)
        ),
    );
    map.insert(
        'm',
        format!(
            "({})[{}]({})",
            number(22),
            from_string(&map, "toString"),
            number(23)
        ),
    );

    // The game plan now is to get the `Function` constructor
    let function_constructor = format!("(()=>{{}})[{}]", from_string(&map, "constructor"));
    map.insert(
        'C',
        format!(
            "({}({})()({}))[{}]",
            function_constructor,
            from_string(&map, "return escape"),
            map.get(&'\\').expect("We set this using regex"),
            number(2)
        ),
    );

    map
}

// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   // Name of the person to greet
   #[arg(short, long)]
   input: String,

   // Number of times to greet
   #[arg(short, long)]
   output: String,
}

fn main() -> std::io::Result<()> {
    let map = build_char_map();

    let args = Args::parse();

    let input_path = args.input;
    let output_path = args.output;
    
    let input_code =
        fs::read_to_string(input_path).expect("Should have been able to read the file");

    let output_code = compile(&map, &input_code);

    let mut file = fs::File::create(output_path)?;
    file.write_all(output_code.as_bytes())
        .expect("Unsuccessful write?");

    Ok(())
}
