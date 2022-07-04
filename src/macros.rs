// current implementation uses extra data to associate keys and values in $const_name
// should be possible to remove this, but the saving might not be worth the explicitness.
macro_rules! parallel_enum_values {
    (($enum_name:ident, $const_name:ident, $const_type:ty $(,)?) $($name:ident -> $value:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $enum_name {
            $($name,)*
        }

        pub const $const_name: &'static [($enum_name, &'static $const_type)] = &[$(($enum_name::$name, $value),)*];
    };
}

pub(crate) use parallel_enum_values;
