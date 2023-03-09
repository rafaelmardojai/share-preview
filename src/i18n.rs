use gettextrs::gettext;

/// Replace variables in the given string with the given dictionary.
///
/// Taken from Fractal https://gitlab.gnome.org/GNOME/fractal/-/blob/main/src/utils/mod.rs
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
pub fn freplace(s: String, args: &[(&str, &str)]) -> String {
    let mut s = s;

    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }

    s
}

/// Like `gettext`, but replaces named variables with the given dictionary.
///
/// Taken from Fractal: https://gitlab.gnome.org/GNOME/fractal/-/blob/main/src/i18n.rs
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
pub fn gettext_f(msgid: &str, args: &[(&str, &str)]) -> String {
    let s = gettext(msgid);
    freplace(s, args)
}
