#[macro_use]
extern crate tenjin;

use std::io;
use tenjin::*;

struct Context<'a> {
    header: &'a str,
    people: &'a [User<'a>],
}

struct User<'a> {
    first:  &'a str,
    last:   &'a str,
    weight: usize,
    html:   &'a str,
}

context! {
    self: ('a) Context<'a> {
        header => @raw  self.header,
        people => @iter self.people, // Will NOT be escaped.
    }

    self: ('a) User<'a> {
        name => @{
            first => self.first,
            last => self.last,
        },
        weight => self.weight,
        html => self.html, // WILL be escaped.
    }
}

fn main() {
    let mut tenjin = Tenjin::empty();

    tenjin.register("test", Template::compile("
        { header }

        First Name: {{ person.name.first }}
        Last Name:  {{ person.name.last }}
        Weight:     {{ person.weight }} kg
        { for person in people }
            First Name: { person.name.first }
            Last Name:  { person.name.last }
            Weight:     { person.weight } kg
            Fav. HTML:  { person.html }
        { end }
    ").unwrap());

    let data = Context {
        header: "<h1>Attack on Titan Registry???</h1>",
        people: &[
            User {
                first: "Eren",
                last: "Jaeger",
                weight: 63,
                html: "<strong>"
            },
            User {
                first: "Mikasa",
                last: "Ackerman",
                weight: 68,
                html: "<em></em>"
            },
            User {
                first: "Armin",
                last: "Arlert",
                weight: 55,
                html: "<pre>...</pre>"
            },
        ],
    };

    let output   = io::stdout();
    let template = tenjin.get("test").unwrap();

    tenjin.render(template, &data, &mut output.lock()).unwrap();
}
