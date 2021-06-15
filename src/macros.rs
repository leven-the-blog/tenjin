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
        fn truthy(&$self, path: $crate::path::Path) -> bool {
            let mut parts = path.parts();

            if let Some(part) = parts.next() {
                context! {
                    $self path part parts
                    __truthy_dict__ $($body)*
                }
            } else {
                // All maps are truthy.
                true
            }
        }

        fn inject(&$self, path: $crate::path::Path, sink: &mut ZZZ)
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

        fn iterate(&$self, path: $crate::path::Path, mut chomp: $crate::render::Chomp<ZZZ>)
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
      __inject_dict__ $key:ident => @{ $($val:tt)* }, $($body:tt)*
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
      __inject_dict__ $key:ident => @raw $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            $crate::Raw($val).inject($parts.as_path(), $sink)
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
            $val.inject($parts.as_path(), $sink)
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
      __iterate_dict__ $key:ident => @{ $($val:tt)* }, $($body:tt)*
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
      __iterate_dict__ $key:ident => @raw $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            $crate::Raw($val).iterate($parts.as_path(), $chomp)
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
            $val.iterate($parts.as_path(), $chomp)
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

    // TRUTHY

    ( $self:ident $path:ident $part:ident $parts:ident
      __truthy_dict__ $key:ident => @{ $($val:tt)* }, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            if let Some(part) = $parts.next() {
                context! {
                    $self $path part $parts
                    __truthy_dict__ $($val)*
                }
            } else {
                // All maps are truthy.
                true
            }
        } else {
            context! {
                $self $path $part $parts
                __truthy_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident
      __truthy_dict__ $key:ident => @iter $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            // All lists are truthy.
            $parts.next().is_none()
        } else {
            context! {
                $self $path $part $parts
                __truthy_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident
      __truthy_dict__ $key:ident => @raw $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            //TODO: This should become unnecessary after we un-traitify this object.
            (&$crate::Raw($val) as &$crate::Context<ZZZ>).truthy($parts.as_path())
        } else {
            context! {
                $self $path $part $parts
                __truthy_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident
      __truthy_dict__ $key:ident => $val:expr, $($body:tt)*
    ) => {
        if stringify!($key) == $part {
            //TODO: This should become unnecessary after we un-traitify this object.
            (&$val as &$crate::Context<ZZZ>).truthy($parts.as_path())
        } else {
            context! {
                $self $path $part $parts
                __truthy_dict__ $($body)*
            }
        }
    };

    ( $self:ident $path:ident $part:ident $parts:ident
      __truthy_dict__
    ) => {
        false
    };
}
