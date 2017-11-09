# tenjin

[![docs.rs](https://docs.rs/tenjin/badge.svg)](https://docs.rs/tenjin) [![crates.io](https://img.shields.io/crates/v/tenjin.svg)](https://crates.io/crates/tenjin)

A dynamic template engine that is fast, and makes zero allocations when rendering.

## Examples

See the [examples directory](https://github.com/quadrupleslap/tenjin/tree/master/examples) for examples!

## Template Syntax

```
{ for item in something.something.items } ... { end }
{ include template_name }
{ something.something.item }
```

To escape `{` and `}`, use `{{` and `}}`, respectively.

## Macro Syntax

The macro is used so that you can pass your structs in as data to your templates.

```rust
context! {
    self: (TYPE PARAMETERS) TYPE {
        key1 => self.value,
        key2 => @iter self.iterable,
        key3 => @raw self.html,
        key4 => @{
            key5 => self.another_value,
            ...
        },
        ...
    }
}
```

See the [macros example](https://github.com/quadrupleslap/tenjin/blob/master/examples/macros.rs) if you're still not clear how this works. Also, note that all values must be contexts, too, and that at the bottom, strings, integers, etc. are the primitive contexts - it's contexts all the way down.

## Contributing

- Features will be added as they are needed. If you think something is missing, please open an issue!
- If you have time to contribute then please do!

## Why does this exist?

I couldn't find a template engine for Rust that:
- was dynamic, allowing for changes to the templates without recompiling the entire program; and
- allowed for structural sharing between the contexts of simultaneous renders.
