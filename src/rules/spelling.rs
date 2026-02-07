// SPDX-FileCopyrightText: 2026 Sébastien Helleu <flashcode@flashtux.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use crate::checker::Checker;
use crate::diagnostic::Severity;
use crate::po::entry::Entry;
use crate::rules::rule::RuleChecker;
use crate::words::WordPos;

pub struct SpellingIdRule {}

impl RuleChecker for SpellingIdRule {
    fn name(&self) -> &'static str {
        "spelling-id"
    }

    fn is_default(&self) -> bool {
        false
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    /// Check spelling in the source stirng (English).
    ///
    /// Wrong entry:
    /// ```text
    /// msgid "this is a tyypo"
    /// msgstr "ceci est une faute"
    /// ```
    ///
    /// Correct entry:
    /// ```text
    /// msgid "this is a typo"
    /// msgstr "ceci est une faute"
    /// ```
    ///
    /// Diagnostics reported with severity [`warning`](Severity::Info):
    /// - `misspelled words in source: xxx`
    fn check_msg(&self, checker: &mut Checker, entry: &Entry, msgid: &str, msgstr: &str) {
        let mut misspelled_words: Vec<&str> = Vec::new();
        let mut hash_words: HashSet<&str> = HashSet::new();
        let mut pos_words = Vec::new();
        if let Some(dict) = &checker.dict_id {
            for (start, end) in WordPos::new(msgid, &entry.format) {
                let word = &msgid[start..end];
                if hash_words.contains(word) {
                    pos_words.push((start, end));
                } else if !dict.check(word) {
                    misspelled_words.push(word);
                    hash_words.insert(word);
                    pos_words.push((start, end));
                }
            }
        }
        if !misspelled_words.is_empty() {
            misspelled_words.sort_unstable();
            checker.report_msg(
                entry,
                format!(
                    "misspelled words in source: {}",
                    misspelled_words.join(", ")
                ),
                msgid,
                &pos_words,
                msgstr,
                &[],
            );
        }
    }
}

pub struct SpellingStrRule {}

impl RuleChecker for SpellingStrRule {
    fn name(&self) -> &'static str {
        "spelling-str"
    }

    fn is_default(&self) -> bool {
        false
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    /// Check spelling in the translated string (using language detected in PO file).
    ///
    /// Wrong entry:
    /// ```text
    /// msgid "this is a typo"
    /// msgstr "ceci est une fôte"
    /// ```
    ///
    /// Correct entry:
    /// ```text
    /// msgid "this is a typo"
    /// msgstr "ceci est une faute"
    /// ```
    ///
    /// Diagnostics reported with severity [`warning`](Severity::Info):
    /// - `misspelled words in translation: xxx`
    fn check_msg(&self, checker: &mut Checker, entry: &Entry, msgid: &str, msgstr: &str) {
        let mut misspelled_words: Vec<&str> = Vec::new();
        let mut hash_words: HashSet<&str> = HashSet::new();
        let mut pos_words = Vec::new();
        if let Some(dict) = &checker.dict_str {
            for (start, end) in WordPos::new(msgstr, &entry.format) {
                let word = &msgstr[start..end];
                if hash_words.contains(word) {
                    pos_words.push((start, end));
                } else if !dict.check(word) {
                    misspelled_words.push(word);
                    hash_words.insert(word);
                    pos_words.push((start, end));
                }
            }
        }
        if !misspelled_words.is_empty() {
            misspelled_words.sort_unstable();
            checker.report_msg(
                entry,
                format!(
                    "misspelled words in translation: {}",
                    misspelled_words.join(", ")
                ),
                msgid,
                &[],
                msgstr,
                &pos_words,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        args::DEFAULT_LANG_ID, diagnostic::Diagnostic, dict::get_dict, rules::rule::Rules,
    };

    fn check_spelling(content: &str) -> Vec<Diagnostic> {
        let rules = Rules::new(vec![
            Box::new(SpellingIdRule {}),
            Box::new(SpellingStrRule {}),
        ]);
        let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_dir.push("resources/test");
        let dict_id = get_dict(test_dir.as_path(), DEFAULT_LANG_ID).unwrap();
        let mut checker = Checker::new(content.as_bytes(), &rules)
            .with_path_dicts(test_dir.as_path())
            .with_dict_id(Some(&dict_id));
        checker.do_all_checks();
        checker.diagnostics
    }

    #[test]
    fn test_spelling_ok() {
        let diags = check_spelling(
            r#"
msgid ""
msgstr "Language: fr\n"

msgid "tested"
msgstr "testé"
"#,
        );
        assert!(diags.is_empty());
    }

    #[test]
    fn test_spelling_error() {
        let diags = check_spelling(
            r#"
msgid ""
msgstr "Language: fr\n"

msgid "this is a tyypo"
msgstr "ceci est une fôte"
"#,
        );
        assert_eq!(diags.len(), 2);
        let diag = &diags[0];
        assert_eq!(diag.severity, Severity::Info);
        assert_eq!(
            diag.message,
            "misspelled words in source: a, is, this, tyypo"
        );
        let diag = &diags[1];
        assert_eq!(diag.severity, Severity::Info);
        assert_eq!(
            diag.message,
            "misspelled words in translation: ceci, est, fôte, une"
        );
    }
}
