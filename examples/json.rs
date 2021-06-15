#[cfg(feature = "serde_json")]
#[macro_use]
extern crate serde_json;
extern crate tenjin;

#[cfg(feature = "serde_json")]
fn main() {
    use std::io;
    use tenjin::*;

    let mut tenjin = Tenjin::empty();

    tenjin.register(
        "test",
        Template::compile(
            "
        First Name: {{ person.name.first }}
        Last Name:  {{ person.name.last }}
        Weight:     {{ person.weight }} kg
        { for person in people }
            First Name: { person.name.first }
            Last Name:  { person.name.last }
            Weight:     { person.weight } kg
        { end }
    ",
        )
        .unwrap(),
    );

    let data = json!({
        "people": [{
            "name": { "first": "Eren", "last": "Jaeger" },
            "weight": 63
        }, {
            "name": { "first": "Mikasa", "last": "Ackerman" },
            "weight": 68
        }, {
            "name": { "first": "Armin", "last": "Arlert" },
            "weight": 55
        }]
    });

    let output = io::stdout();
    let template = tenjin.get("test").unwrap();

    tenjin.render(template, &data, &mut output.lock()).unwrap();
}

#[cfg(not(feature = "serde_json"))]
fn main() {
    println!("Please enable `serde_json` integration to run this example.");
}
