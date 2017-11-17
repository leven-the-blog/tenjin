# tenjin

[![docs.rs](https://docs.rs/tenjin/badge.svg)](https://docs.rs/tenjin) [![crates.io](https://img.shields.io/crates/v/tenjin.svg)](https://crates.io/crates/tenjin)

## Buzzwords

- **dynamic**
- **zero-allocation** in render
- **logic-less**
- enables **structural sharing**

## Template Syntax

```
{ if path.to.item } ... { end }
{ for item in path.to.items } ... { end }
{ include template_name }
{ path.to.item }
```

To escape `{` and `}`, use `{{` and `}}`, respectively.

## Macro Syntax

A macro can be used so that you can pass your own structs in as data to your templates.

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

You might also want to see the [macros example](https://github.com/quadrupleslap/tenjin/blob/master/examples/macros.rs). Note that these "contexts" are composable.

## Truthiness

1. All undefined values are falsey.
2. Objects, maps and arrays are truthy.
3. Booleans evaluate to their own value.
4. Integers are truthy iff they are non-zero.
5. Strings are truthy iff they are non-empty.
6. Try to make truthiness as unsurprising as possible.

## Contributing

Features will be added as they are needed. If you think something is missing, please open an issue!
