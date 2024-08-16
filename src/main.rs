use clap::Parser;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::{fs::read_to_string, mem::swap, path::PathBuf};

#[derive(Parser, Debug)]
struct Args {
    grammar: PathBuf,
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

#[derive(Debug, Deserialize)]
struct Rule {
    from: String,
    to: String,
}

fn main() {
    let args = Args::parse();

    let grammar = serde_json::from_str::<Grammar>(&read_to_string(&args.grammar).unwrap()).unwrap();

    let mut words = FxHashSet::default();
    words.insert(grammar.start_symbol.clone());
    let mut new_words = FxHashSet::default();
    let mut words_rule_applied = FxHashSet::default();
    let mut results = FxHashSet::default();

    let mut iters = 0;
    while iters < args.max_iters.unwrap_or(usize::MAX) && !words.is_empty() {
        for word in &words {
            for rule in &grammar.rules {
                apply_rule(&mut words_rule_applied, rule, word);

                for new_word in words_rule_applied.drain() {
                    if is_only_terms(&new_word, &grammar) {
                        results.insert(new_word);
                    } else {
                        new_words.insert(new_word);
                    }
                }
            }
        }

        swap(&mut words, &mut new_words);
        new_words.clear();

        iters += 1;
    }

    dbg!(iters);
    dbg!(results);
}

fn is_only_terms(s: &str, grammar: &Grammar) -> bool {
    s.chars().all(|c| {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        grammar.term_symbols.iter().any(|s| s == encoded)
    })
}

fn apply_rule(words_rule_applied: &mut FxHashSet<String>, rule: &Rule, word: &str) {
    words_rule_applied.extend(word.match_indices(&rule.from).map(|(i, _)| i).map(|index| {
        let mut new_string = String::with_capacity(word.len() + rule.to.len() - rule.from.len());
        new_string.push_str(&word[..index]);
        new_string.push_str(&rule.to);
        new_string.push_str(&word[index + rule.from.len()..]);
        new_string
    }));
}
