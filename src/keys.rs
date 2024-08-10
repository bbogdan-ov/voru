/// Will generate something like this:
///
/// From:
///
/// ```no_run
/// key_match! {
///     config, key,
///
///     keymap_name => Msg::SomeMessage,
///     another_keymap => Msg::AnotherMessage;
///
///     else { /* Do something... */ }
/// };
/// ```
///
/// To:
///
/// ```no_run
/// if false { unreachable!() }
/// else if config.keys.keymap_name.as_ref().is_some_and(|k| k.contains(&key)) { Msg::SomeMessage }
/// else if config.keys.another_keymap.as_ref().is_some_and(|k| k.contains(&key)) { Msg::AnotherMessage }
/// // ...
/// else { /* Do something... */ }
/// ```
#[macro_export]
macro_rules! match_keys {
    ($config:ident, $key:ident, $($keymap:ident => $do:expr$(,)?)*; else $otherwise:block) => {
        if false { unreachable!() }
        $(else if $config.keys.$keymap.as_ref().is_some_and(|k| k.contains(&$key)) { $do })*
        else $otherwise
    };
}

#[macro_export]
macro_rules! key {
    ($char:literal) => { Key(KeyMod::Any, KeyCode::Char($char)) };
    ($key:ident) => { Key(KeyMod::Any, KeyCode::$key) };
    (Shift + $key:ident) => { Key(KeyMod::Shift, KeyCode::$key) };
    (Ctrl + $char:literal) => { Key(KeyMod::Ctrl, KeyCode::Char($char)) };
    (Ctrl + $key:ident) => { Key(KeyMod::Ctrl, KeyCode::$key) };
}
