use yaml_rust::YamlLoader;

use crate::error::Error;

pub enum Extends {
    Existence,
    Substitution,
    Occurrence,
    Repetition,
    Consistency,
    Conditional,
    Capitalization,
    Metric,
    Spelling,
    Sequence,
    Script,
    Invalid,
}

pub struct Rule {
    pub extends: Extends,
}

impl Rule {
    pub(crate) fn new(rule_path: &str) -> Result<Rule, Error> {
        let src = std::fs::read_to_string(rule_path)?;
        match YamlLoader::load_from_str(&src) {
            Ok(docs) => {
                let doc = &docs[0];
                let extends = match doc["extends"].as_str().unwrap_or("invalid") {
                    "existence" => Extends::Existence,
                    "substitution" => Extends::Substitution,
                    "occurrence" => Extends::Occurrence,
                    "repetition" => Extends::Repetition,
                    "consistency" => Extends::Consistency,
                    "conditional" => Extends::Conditional,
                    "capitalization" => Extends::Capitalization,
                    "metric" => Extends::Metric,
                    "spelling" => Extends::Spelling,
                    "sequence" => Extends::Sequence,
                    "script" => Extends::Script,
                    _ => Extends::Invalid,
                };
                Ok(Rule { extends })
            }
            Err(_) => Ok(Rule {
                extends: Extends::Invalid,
            }),
        }
    }

    /// Returns the documentation for a given token, if it exists.
    pub(crate) fn token_info(&self, token: &str) -> Option<String> {
        match self.extends {
            Extends::Existence => self.existence(token),
            Extends::Substitution => self.substitution(token),
            Extends::Occurrence => self.occurrence(token),
            Extends::Repetition => self.repetition(token),
            Extends::Consistency => self.consistency(token),
            Extends::Conditional => self.conditional(token),
            Extends::Capitalization => self.capitalization(token),
            Extends::Metric => self.metric(token),
            Extends::Spelling => self.spelling(token),
            Extends::Sequence => self.sequence(token),
            Extends::Script => self.script(token),
            Extends::Invalid => None,
        }
    }

    fn common(&self, token: &str, example: String) -> Option<String> {
        match token {
            "extends" => {
                let docs = include_str!("../doc/yml/extends.md").to_string();
                Some(format!("{}\n\n## Example\n\n{}", docs, example))
            }
            "message" => Some(include_str!("../doc/yml/message.md").to_string()),
            "level" => Some(include_str!("../doc/yml/level.md").to_string()),
            "scope" => Some(include_str!("../doc/yml/scope.md").to_string()),
            "link" => Some(include_str!("../doc/yml/link.md").to_string()),
            "limit" => Some(include_str!("../doc/yml/limit.md").to_string()),
            "action" => Some(include_str!("../doc/yml/action.md").to_string()),
            _ => None,
        }
    }

    fn existence(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/existence/example.md").to_string();
        match key {
            "append" => Some(include_str!("../doc/yml/existence/append.md").to_string()),
            "ignorecase" => Some(include_str!("../doc/yml/existence/ignorecase.md").to_string()),
            "nonword" => Some(include_str!("../doc/yml/existence/nonword.md").to_string()),
            "raw" => Some(include_str!("../doc/yml/existence/raw.md").to_string()),
            "tokens" => Some(include_str!("../doc/yml/existence/tokens.md").to_string()),
            "exceptions" => Some(include_str!("../doc/yml/existence/exceptions.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn substitution(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/substitution/example.md").to_string();
        match key {
            "append" => Some(include_str!("../doc/yml/substitution/append.md").to_string()),
            "ignorecase" => Some(include_str!("../doc/yml/substitution/ignorecase.md").to_string()),
            "nonword" => Some(include_str!("../doc/yml/substitution/nonword.md").to_string()),
            "exceptions" => Some(include_str!("../doc/yml/substitution/exceptions.md").to_string()),
            "swap" => Some(include_str!("../doc/yml/substitution/swap.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn occurrence(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/occurrence/example.md").to_string();
        match key {
            "min" => Some(include_str!("../doc/yml/occurrence/min.md").to_string()),
            "max" => Some(include_str!("../doc/yml/occurrence/max.md").to_string()),
            "token" => Some(include_str!("../doc/yml/occurrence/token.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn repetition(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/repetition/example.md").to_string();
        match key {
            "alpha" => Some(include_str!("../doc/yml/repetition/alpha.md").to_string()),
            "tokens" => Some(include_str!("../doc/yml/repetition/tokens.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn consistency(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/consistency/example.md").to_string();
        match key {
            "either" => Some(include_str!("../doc/yml/consistency/either.md").to_string()),
            "nonword" => Some(include_str!("../doc/yml/consistency/nonword.md").to_string()),
            "ignorecase" => Some(include_str!("../doc/yml/consistency/ignorecase.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn conditional(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/conditional/example.md").to_string();
        match key {
            "first" => Some(include_str!("../doc/yml/conditional/first.md").to_string()),
            "second" => Some(include_str!("../doc/yml/conditional/second.md").to_string()),
            "ignorecase" => Some(include_str!("../doc/yml/conditional/ignorecase.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn capitalization(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/capitalization/example.md").to_string();
        match key {
            "exceptions" => {
                Some(include_str!("../doc/yml/capitalization/exceptions.md").to_string())
            }
            "match" => Some(include_str!("../doc/yml/capitalization/match.md").to_string()),
            "style" => Some(include_str!("../doc/yml/capitalization/style.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn metric(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/metric/example.md").to_string();
        match key {
            "formula" => Some(include_str!("../doc/yml/metric/formula.md").to_string()),
            "condition" => Some(include_str!("../doc/yml/metric/condition.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn spelling(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/spelling/example.md").to_string();
        match key {
            "append" => Some(include_str!("../doc/yml/spelling/append.md").to_string()),
            "custom" => Some(include_str!("../doc/yml/spelling/custom.md").to_string()),
            "dicpath" => Some(include_str!("../doc/yml/spelling/dicpath.md").to_string()),
            "dictionaries" => Some(include_str!("../doc/yml/spelling/dictionaries.md").to_string()),
            "filters" => Some(include_str!("../doc/yml/spelling/filters.md").to_string()),
            "ignore" => Some(include_str!("../doc/yml/spelling/ignore.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn sequence(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/sequence/example.md").to_string();
        match key {
            "ignorecase" => Some(include_str!("../doc/yml/sequence/ignorecase.md").to_string()),
            "tokens" => Some(include_str!("../doc/yml/sequence/tokens.md").to_string()),
            _ => self.common(key, example),
        }
    }

    fn script(&self, key: &str) -> Option<String> {
        let example = include_str!("../doc/yml/script/example.md").to_string();
        match key {
            "script" => Some(include_str!("../doc/yml/script/script.md").to_string()),
            _ => self.common(key, example),
        }
    }
}
