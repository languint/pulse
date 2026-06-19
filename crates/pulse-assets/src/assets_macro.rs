#[macro_export]
macro_rules! assets {
    (
        icons {
            $(
                $name:ident => $path:literal
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Icon {
            $(
                $name,
            )*
        }

        impl Icon {
            pub fn path(&self) -> &'static str {
                match self {
                    $(
                        Self::$name => concat!("icons/", $path),
                    )*
                }
            }

            pub fn bytes(&self) -> &'static [u8] {
                match self {
                    $(
                        Self::$name => include_bytes!($path),
                    )*
                }
            }
        }

        pub fn get(path: &str) -> Option<&'static [u8]> {
            match path {
                $(
                    concat!("icons/", $path) => Some(include_bytes!($path)),
                )*
                _ => None,
            }
        }

        pub fn list() -> &'static [&'static str] {
            &[
                $(
                    concat!("icons/", $path),
                )*
            ]
        }

    };
}
