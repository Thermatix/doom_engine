#[macro_export]
macro_rules! try_outer_to_inner {
    ($a: ident, $b: ident, $e: ident, $f: ident) => {
        // impl<'i> TryFrom<&'i $a> for &'i $b {

        //     type Error = $e;
        
        //     fn try_from(outer: &'i $a) -> Result<Self, Self::Error> {
        //         match outer {
        //             $a::$b(inner) => Ok(&inner),
        //             _ => Err($e::$f(stringify!($a).to_string(), stringify!($b).to_string()))
        //         }
        //     }
        // }

        impl<'i> From<&'i $a> for &'i $b {
            fn from(outer: &'i $a) -> Self {
                match outer {
                    $a::$b(inner) => &inner,
                    _ => panic!("Tried to unwrap into wrong type")
                }
            }
        }

        impl<'i> From<&'i $a> for $b {
            fn from(outer: &'i $a) -> Self {
                match outer {
                    $a::$b(inner) => inner.clone(),
                    _ => panic!("Tried to unwrap into wrong type")
                }
            }
        }

        impl From<$a> for $b {
            fn from(outer: $a) -> Self {
                match outer {
                    $a::$b(inner) => inner.clone(),
                    _ => panic!("Tried to unwrap into wrong type")
                }
            }
        }
    };
}