//TODO: Macros 2.0 and.or somebody who actually knows macros should improve this.

#[macro_export]
macro_rules! context {
    // MAIN
    
    () => {};

    (
        $self:ident : ($($params:tt)+) $type:ty {
            $($body:tt)*
        }

        $($rest:tt)*

    ) => {
        impl<$($params)*, ZZZ: ::std::io::Write> $crate::Context<ZZZ> for $type {
            #![allow(dead_code, unused_variables, unused_mut)]

            context! {
                $self
                __fns__ $($body)*
            }
        }

        context! { $($rest)* }
    };

    (
        $self:ident : $type:ty {
            $($body:tt)*
        }

        $($rest:tt)*

    ) => {
        impl<ZZZ: ::std::io::Write> $crate::Context<ZZZ> for $type {
            #![allow(dead_code, unused_variables, unused_mut)]

            context! {
                $self
                __fns__ $($body)*
            }
        }

        context! { $($rest)* }
    };

    // TRAIT FUNCTIONS

    ( $self:ident
      __fns__ $($body:tt)*
    ) => {
        fn inject(&$self, path: $crate::Path, sink: &mut ZZZ)
            -> $crate::Result<()>
        {
            let mut parts = path.parts();

            if let Some(part) = parts.next() {
                context! {
                    $self path part parts sink
                    __inject_dict__ $($body)*
                }
            } else {
                Err($crate::Error::NotInjectable(path.to_owned()))
            }
        }

        fn iterate(&$self, path: $crate::Path, mut chomp: $crate::Chomp<ZZZ>)
            -> $crate::Result<()>
        {
            let mut parts = path.parts();

            if let Some(part) = parts.next() {
                context! {
                    $self path part parts chomp
                    __iterate_dict__ $($body)*
                }
            } else {
                Err($crate::Error::NotIterable(path.to_owned()))
            }
        }
    };

    // INJECTION

    ( $self:ident $path:ident $part:ident $parts:ident $sink:ident
      __inject_dict__ $key:ident => #{ $($val:tt)* }, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            if let Some(part) = $parts.next() {
                context! {
                    $self $path part $parts $sink
                    __inject_dict__ $($val)*
                }
            } else {
                Err($crate::Error::NotInjectable($path.to_owned()))
            }
        } else {
            context! {
                $self $path $part $parts $sink
                __inject_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident $sink:ident
      __inject_dict__ $key:ident => @iter $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            Err(match $parts.next() {
                None => $crate::Error::NotInjectable($path.to_owned()),
                Some(_) => $crate::Error::Undefined($path.to_owned())
            })
        } else {
            context! {
                $self $path $part $parts $sink
                __inject_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident $sink:ident
      __inject_dict__ $key:ident => $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            match $parts.next() {
                None => $val.inject($parts.as_path(), $sink),
                Some(_) => Err($crate::Error::Undefined($path.to_owned()))
            }
        } else {
            context! {
                $self $path $part $parts $sink
                __inject_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident $sink:ident
      __inject_dict__
    ) => {
        Err($crate::Error::Undefined($path.to_owned()))
    };

    // ITERATION

    ( $self:ident $path:ident $part:ident $parts:ident $chomp:ident
      __iterate_dict__ $key:ident => #{ $($val:tt)* }, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            if let Some(part) = $parts.next() {
                context! {
                    $self $path part $parts $chomp
                    __iterate_dict__ $($val)*
                }
            } else {
                Err($crate::Error::NotIterable($path.to_owned()))
            }
        } else {
            context! {
                $self $path $part $parts $chomp
                __iterate_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident $chomp:ident
      __iterate_dict__ $key:ident => @iter $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            match $parts.next() {
                None => {
                    for item in $val {
                        $chomp.chomp(item)?;
                    }
                    Ok(())
                }
                Some(_) => Err($crate::Error::Undefined($path.to_owned()))
            }
        } else {
            context! {
                $self $path $part $parts $chomp
                __iterate_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident $chomp:ident
      __iterate_dict__ $key:ident => $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            match $parts.next() {
                None => $val.iterate($parts.as_path(), $chomp),
                Some(_) => Err($crate::Error::Undefined($path.to_owned()))
            }
        } else {
            context! {
                $self $path $part $parts $chomp
                __iterate_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident $chomp:ident
      __iterate_dict__
    ) => {
        Err($crate::Error::Undefined($path.to_owned()))
    };
}

//TODO: HTML Escaping.

struct Todo<'a> {
    a: &'a str,
    b: &'a str,
}

context! {
    self: ('a) Todo<'a> {
        a => self.a,
        b => self.b,
        c => @iter &["hello", "?"][..],
        d => #{
            a => self.a,
            b => self.b,
        },
    }
}
