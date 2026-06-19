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
        pub enum IconName {
            $(
                $name,
            )*
        }

        impl IconName {
            pub const fn path(&self) -> &'static str {
                match self {
                    $(
                        Self::$name => concat!("icons/", $path),
                    )*
                }
            }

            pub const fn bytes(&self) -> &'static [u8] {
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

        pub const fn list() -> &'static [&'static str] {
            &[
                $(
                    concat!("icons/", $path),
                )*
            ]
        }

    };
}
