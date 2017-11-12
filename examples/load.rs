extern crate tenjin;

use tenjin::Tenjin;

fn main() {
    let mut path = "templates".into();
    println!("Loading all HTML files from the `templates` directory...");
    println!("{:?}", Tenjin::new(&mut path).unwrap());
}
