use aho_corasick::AhoCorasick;
use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::{fs::read_to_string, mem::swap, path::PathBuf, sync::Mutex};

#[derive(Parser, Debug)]
struct Args {
    /// The path to the grammar json file
    grammar: PathBuf,
    /// Maximum number of iterations
    max_iters: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct Grammar {
    #[allow(dead_code)]
    var_symbols: Vec<String>,
    term_symbols: Vec<String>,
    start_symbol: String,
    rules: Vec<Rule>,
}

#[doc(hidden)]
#[derive(Debug, Deserialize)]
struct IntermediateRule {
    from: String,
    to: String,
}

#[derive(Debug)]
struct Rule {
    from: String,
    to: String,

    aho: AhoCorasick,
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let intermediate_rule: IntermediateRule = Deserialize::deserialize(deserializer)?;
        let aho = AhoCorasick::new([&intermediate_rule.from]).unwrap();

        Ok(Rule {
            from: intermediate_rule.from,
            to: intermediate_rule.to,

            aho,
        })
    }
}

fn main() {
    let args = Args::parse();

    let grammar = serde_json::from_str::<Grammar>(&read_to_string(&args.grammar).unwrap()).unwrap();

    let mut words = FxHashSet::default();
    words.insert(grammar.start_symbol.clone());
    let new_words = Mutex::new(FxHashSet::default());
    let results = Mutex::new(FxHashSet::<String>::default());

    let mut iters = 0;
    while iters < args.max_iters.unwrap_or(usize::MAX) && !words.is_empty() {
        grammar.rules.par_iter().for_each(|rule| {
            let mut words_rule_applied = Vec::new();
            for word in &words {
                apply_rule(&mut words_rule_applied, rule, word);
            }

            for word in words_rule_applied {
                if is_only_terms(&word, &grammar) {
                    results.lock().unwrap().insert(word);
                } else {
                    new_words.lock().unwrap().insert(word);
                }
            }
        });

        words.clear();
        swap(&mut words, &mut new_words.lock().unwrap());

        iters += 1;
    }

    dbg!(iters);
    dbg!(results.into_inner().unwrap());
}

fn is_only_terms(s: &str, grammar: &Grammar) -> bool {
    s.chars().all(|c| {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        grammar.term_symbols.iter().any(|s| s == encoded)
    })
}

fn apply_rule(words_rule_applied: &mut Vec<String>, rule: &Rule, word: &str) {
    words_rule_applied.extend(rule.aho.find_iter(word).map(|mat| {
        let mut new_string = String::with_capacity(word.len() + rule.to.len() - rule.from.len());
        new_string.push_str(&word[..mat.start()]);
        new_string.push_str(&rule.to);
        new_string.push_str(&word[mat.end()..]);
        new_string
    }));
}
