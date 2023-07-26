
#[derive(Debug)]
struct Test {
    pub name: String,
}

fn main() {
    let test = Test { name: "Martin".to_string() };
    let me = &test.name;
    println!("Hello {me}!");
}

