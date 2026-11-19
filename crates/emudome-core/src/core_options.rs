//! Core options: the values the shell answers to `RETRO_ENVIRONMENT_GET_VARIABLE`.
//!
//! The core declares its options (`SET_VARIABLES`, `SET_CORE_OPTIONS*`): key, accepted values,
//! default. The shell brings its own values (`SessionOptions::core_options`, later the
//! settings engine's `cores.*` entries). The answer for a key is the shell's value when the
//! core accepts it, else the core's default. Pure: the C structures are read in `callbacks.rs`.

use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};

/// One option as the core declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OptionDef {
    pub key: String,
    /// The core's own label (`desc`), in English: for logs and the option listing only.
    pub label: String,
    /// Accepted values, in the core's order.
    pub values: Vec<String>,
    pub default: String,
}

impl OptionDef {
    /// Builds a definition; with no declared default, the first value is the default.
    pub fn new(key: String, values: Vec<String>, default: Option<String>) -> Self {
        let default = default
            .filter(|d| !d.is_empty())
            .or_else(|| values.first().cloned())
            .unwrap_or_default();
        Self {
            key,
            label: String::new(),
            values,
            default,
        }
    }

    /// The same definition with the core's label.
    pub fn with_label(mut self, label: Option<String>) -> Self {
        self.label = label.unwrap_or_default();
        self
    }

    /// Parses a v0 variable (`SET_VARIABLES`): `"Description; first|second|third"`,
    /// the first value being the default. None when there is no `; ` separator.
    pub fn from_variable(key: &str, raw: &str) -> Option<Self> {
        let (desc, list) = raw.split_once("; ")?;
        let values: Vec<String> = list.split('|').map(str::to_owned).collect();
        Some(Self::new(key.to_owned(), values, None).with_label(Some(desc.to_owned())))
    }

    fn accepts(&self, value: &str) -> bool {
        self.values.is_empty() || self.values.iter().any(|v| v == value)
    }
}

/// The options of one session: the core's definitions and the shell's values.
#[derive(Debug, Default)]
pub(crate) struct CoreOptions {
    defs: HashMap<String, OptionDef>,
    /// Values chosen by the shell, by key.
    chosen: HashMap<String, String>,
    /// What `GET_VARIABLE` answers, kept as C strings: the core gets pointers into them.
    /// An entry is replaced only when its value changes.
    answers: HashMap<String, CString>,
    /// An answer changed since the core last asked `GET_VARIABLE_UPDATE`.
    updated: bool,
}

impl CoreOptions {
    pub fn new(chosen: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut me = Self {
            chosen: chosen.into_iter().collect(),
            ..Self::default()
        };
        me.refresh();
        me.updated = false;
        me
    }

    /// The core's definitions, sorted by key (the option listing, `Shared::core_options`).
    pub fn declared(&self) -> Vec<crate::session::CoreOptionInfo> {
        let mut out: Vec<_> = self
            .defs
            .values()
            .map(|d| crate::session::CoreOptionInfo {
                key: d.key.clone(),
                label: d.label.clone(),
                values: d.values.clone(),
                default: d.default.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.key.cmp(&b.key));
        out
    }

    /// The core declared its options: they replace the earlier ones.
    pub fn define(&mut self, defs: Vec<OptionDef>) {
        self.defs = defs.into_iter().map(|d| (d.key.clone(), d)).collect();
        for (key, value) in &self.chosen {
            if let Some(def) = self.defs.get(key) {
                if !def.accepts(value) {
                    log::warn!(
                        "core option {key}={value} is not one the core accepts, using {}",
                        def.default
                    );
                }
            }
        }
        self.refresh();
        // Declaring is not a change the core has to re-read.
        self.updated = false;
    }

    /// The shell changes a value while the game runs: the core sees it at its next
    /// `GET_VARIABLE_UPDATE`.
    pub fn set(&mut self, key: String, value: String) {
        self.chosen.insert(key, value);
        self.refresh();
    }

    /// The answer to `GET_VARIABLE`: None when neither the core nor the shell knows the key.
    pub fn answer(&self, key: &str) -> Option<&CStr> {
        self.answers.get(key).map(CString::as_c_str)
    }

    /// The answer to `GET_VARIABLE_UPDATE`; reading it clears it.
    pub fn take_updated(&mut self) -> bool {
        std::mem::take(&mut self.updated)
    }

    fn resolve(&self, key: &str) -> Option<&str> {
        let chosen = self.chosen.get(key).map(String::as_str);
        match (self.defs.get(key), chosen) {
            (Some(def), Some(v)) if def.accepts(v) => Some(v),
            (Some(def), _) => Some(def.default.as_str()),
            // Not declared (yet): the shell's value goes through as it is.
            (None, v) => v,
        }
    }

    fn refresh(&mut self) {
        let keys: HashSet<String> = self
            .defs
            .keys()
            .chain(self.chosen.keys())
            .cloned()
            .collect();
        let mut answers = HashMap::new();
        for key in keys {
            let Some(value) = self.resolve(&key) else {
                continue;
            };
            // A value with a NUL byte cannot reach C: the key is left unanswered.
            let Ok(c) = CString::new(value) else {
                continue;
            };
            match self.answers.remove(&key) {
                Some(old) if old == c => {
                    answers.insert(key, old);
                }
                Some(_) | None => {
                    self.updated = true;
                    answers.insert(key, c);
                }
            }
        }
        if !self.answers.is_empty() {
            self.updated = true; // a key lost its answer
        }
        self.answers = answers;
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreOptions, OptionDef};

    fn s(v: &str) -> String {
        v.to_owned()
    }

    fn def(key: &str, values: &[&str], default: Option<&str>) -> OptionDef {
        OptionDef::new(
            s(key),
            values.iter().map(|v| s(v)).collect(),
            default.map(s),
        )
    }

    fn answer(o: &CoreOptions, key: &str) -> Option<String> {
        o.answer(key).map(|c| c.to_string_lossy().into_owned())
    }

    #[test]
    fn variable_v0_parses_and_first_is_default() {
        let d = OptionDef::from_variable("k", "Renderer; Software|OpenGL").unwrap();
        assert_eq!(d.values, vec![s("Software"), s("OpenGL")]);
        assert_eq!(d.default, "Software");
        assert!(OptionDef::from_variable("k", "no separator").is_none());
    }

    #[test]
    fn missing_or_empty_default_is_first_value() {
        assert_eq!(def("k", &["a", "b"], None).default, "a");
        assert_eq!(def("k", &["a", "b"], Some("")).default, "a");
        assert_eq!(def("k", &["a", "b"], Some("b")).default, "b");
    }

    #[test]
    fn unknown_key_has_no_answer() {
        let mut o = CoreOptions::new([]);
        o.define(vec![def("a", &["x"], None)]);
        assert_eq!(answer(&o, "missing"), None);
    }

    #[test]
    fn default_answers_when_the_shell_says_nothing() {
        let mut o = CoreOptions::new([]);
        o.define(vec![def(
            "citra_graphics_api",
            &["Auto", "Software"],
            Some("Auto"),
        )]);
        assert_eq!(answer(&o, "citra_graphics_api").as_deref(), Some("Auto"));
    }

    #[test]
    fn shell_value_wins_when_accepted() {
        let mut o = CoreOptions::new([(s("citra_graphics_api"), s("Software"))]);
        o.define(vec![def(
            "citra_graphics_api",
            &["Auto", "Vulkan", "OpenGL", "Software"],
            Some("auto"),
        )]);
        assert_eq!(
            answer(&o, "citra_graphics_api").as_deref(),
            Some("Software")
        );
    }

    #[test]
    fn refused_shell_value_falls_back_to_default() {
        let mut o = CoreOptions::new([(s("k"), s("software"))]);
        o.define(vec![def("k", &["Auto", "Software"], None)]);
        assert_eq!(answer(&o, "k").as_deref(), Some("Auto"));
    }

    #[test]
    fn undeclared_shell_value_goes_through() {
        // A core may ask before declaring anything (or never declare).
        let o = CoreOptions::new([(s("k"), s("v"))]);
        assert_eq!(answer(&o, "k").as_deref(), Some("v"));
    }

    #[test]
    fn update_flag_only_after_a_change() {
        let mut o = CoreOptions::new([(s("k"), s("a"))]);
        o.define(vec![def("k", &["a", "b"], None)]);
        assert!(!o.take_updated());
        o.set(s("k"), s("a"));
        assert!(!o.take_updated(), "same value: nothing to re-read");
        o.set(s("k"), s("b"));
        assert!(o.take_updated());
        assert!(!o.take_updated(), "reading clears the flag");
        assert_eq!(answer(&o, "k").as_deref(), Some("b"));
    }

    #[test]
    fn unchanged_answer_keeps_its_pointer() {
        let mut o = CoreOptions::new([]);
        o.define(vec![def("a", &["x"], None), def("b", &["y", "z"], None)]);
        let before = o.answer("a").unwrap().as_ptr();
        o.set(s("b"), s("z"));
        assert_eq!(o.answer("a").unwrap().as_ptr(), before);
    }
}
