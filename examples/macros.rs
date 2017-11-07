#[macro_use]
extern crate tenjin;

use std::io;
use tenjin::*;

struct Context<'a> {
    people: &'a [User<'a>],
}

struct User<'a> {
    first: &'a str,
    last: &'a str,
    weight: usize,
}

context! {
    self: ('a) User<'a> {
        name => #{
            first => self.first,
            last => self.last,
        },
        weight => self.weight,
    }

    self: ('a) Context<'a> {
        people => @iter self.people,
    }
}

fn main() {
    let mut tenjin = Tenjin::empty();

    tenjin.register("test", Template::compile("
        First Name: {{ person.name.first }}
        Last Name:  {{ person.name.last }}
        Weight:     {{ person.weight }} kg
        { for person in people }
            First Name: { person.name.first }
            Last Name:  { person.name.last }
            Weight:     { person.weight } kg
        { end }
    ").unwrap());

    let data = Context {
        people: &[
            User { first: "Eren",   last: "Jaeger",   weight: 63 },
            User { first: "Mikasa", last: "Ackerman", weight: 68 },
            User { first: "Armin",  last: "Arlert",   weight: 55 },
        ]
    };

    let output   = io::stdout();
    let template = tenjin.get("test").unwrap();

    tenjin.render(template, &data, &mut output.lock()).unwrap();
}
