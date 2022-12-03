use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use regex::Regex;
use reqwest;
use serde::Deserialize;
use tower_lsp::lsp_types::*;

const PKGS: &str = "https://raw.githubusercontent.com/errata-ai/packages/master/library.json";

#[derive(Deserialize, Debug, Clone)]
struct Package {
    name: String,
    description: String,
    homepage: String,
}

pub fn key_to_info(key: &str) -> Option<String> {
    match key {
        "StylesPath" => Some(include_str!("../doc/ini/StylesPath.md").to_string()),
        "MinAlertLevel" => Some(include_str!("../doc/ini/MinAlertLevel.md").to_string()),
        "IgnoredScopes" => Some(include_str!("../doc/ini/IgnoredScopes.md").to_string()),
        "IgnoredClasses" => Some(include_str!("../doc/ini/IgnoredClasses.md").to_string()),
        "SkippedScopes" => Some(include_str!("../doc/ini/SkippedScopes.md").to_string()),
        "WordTemplate" => Some(include_str!("../doc/ini/WordTemplate.md").to_string()),
        "BasedOnStyles" => Some(include_str!("../doc/ini/BasedOnStyles.md").to_string()),
        "BlockIgnores" => Some(include_str!("../doc/ini/BlockIgnores.md").to_string()),
        "TokenIgnores" => Some(include_str!("../doc/ini/TokenIgnores.md").to_string()),
        "Transform" => Some(include_str!("../doc/ini/Transform.md").to_string()),
        "Vocab" => Some(include_str!("../doc/ini/Vocab.md").to_string()),
        "Packages" => Some(include_str!("../doc/ini/Packages.md").to_string()),
        _ => None,
    }
}

pub async fn complete(line: &str, styles: PathBuf) -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    let re = Regex::new(r"\w+\.\w+ =").unwrap();

    if line.contains("BasedOnStyles") {
        completions = get_styles(line, styles);
    } else if line.contains("MinAlertLevel") {
        vec![
            "suggestion".to_string(),
            "warning".to_string(),
            "error".to_string(),
        ]
        .into_iter()
        .for_each(|s| {
            completions.push(CompletionItem {
                label: s.clone(),
                insert_text: Some(s),
                kind: Some(CompletionItemKind::VALUE),
                ..CompletionItem::default()
            })
        });
    } else if line.contains("IgnoredScopes") {
        completions = inline_tags();
    } else if line.contains("SkippedScopes") {
        completions = block_tags();
    } else if re.is_match(line) {
        completions = rule_options();
    } else if line.contains("Vocab") {
        completions = get_vocab(line, styles);
    } else if line.contains("Packages") {
        completions = get_pkgs(line).await;
    }

    completions
}

async fn get_pkgs(line: &str) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    match reqwest::get(PKGS).await {
        Ok(resp) => {
            if resp.status().is_success() {
                let info: Vec<Package> = resp.json().await.unwrap();
                for pkg in info {
                    if !line.contains(pkg.name.as_str()) {
                        completions.push(CompletionItem {
                            label: pkg.name.clone(),
                            insert_text: Some(pkg.name.clone()),
                            kind: Some(CompletionItemKind::VALUE),
                            label_details: Some(CompletionItemLabelDetails {
                                description: Some(pkg.description),
                                ..CompletionItemLabelDetails::default()
                            }),
                            detail: Some("Package".to_string()),
                            preselect: Some(true),
                            ..CompletionItem::default()
                        });
                    }
                }
            }
        }
        Err(_) => (),
    }

    completions
}

fn get_vocab(line: &str, styles: PathBuf) -> Vec<CompletionItem> {
    match fs::read_dir(styles.join("Vocab").as_path()) {
        Ok(paths) => {
            let mut found = vec![];

            let dirs: Vec<PathBuf> = paths
                .into_iter()
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap().path())
                .filter(|r| r.is_dir())
                .collect();

            for path in dirs {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if !line.contains(&name) {
                    found.push(CompletionItem {
                        label: name.clone(),
                        insert_text: Some(name.clone()),
                        kind: Some(CompletionItemKind::VALUE),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: path.display().to_string(),
                        })),
                        detail: Some("Vocab".to_string()),
                        ..CompletionItem::default()
                    });
                }
            }

            found
        }
        Err(_) => vec![],
    }
}

fn rule_options() -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    let options = HashMap::from([
        ("YES", "Enable the given rule in this scope."),
        ("NO", "Disable the given rule in this scope."),
        ("suggestion", "Set the severity to 'suggestion'."),
        ("warning", "Set the severity to 'warning'."),
        ("error", "Set the severity to 'error'."),
    ]);

    for (key, value) in options {
        completions.push(CompletionItem {
            label: key.to_string(),
            insert_text: Some(key.to_string()),
            kind: Some(CompletionItemKind::VALUE),
            label_details: Some(CompletionItemLabelDetails {
                description: Some(format!("{}", value)),
                ..CompletionItemLabelDetails::default()
            }),
            ..CompletionItem::default()
        });
    }

    completions
}

fn inline_tags() -> Vec<CompletionItem> {
    vec![
        "small".to_string(),
        "abbr".to_string(),
        "em".to_string(),
        "kbd".to_string(),
        "tt".to_string(),
        "code".to_string(),
        "b".to_string(),
        "i".to_string(),
    ]
    .into_iter()
    .map(|s| CompletionItem {
        label: s.clone(),
        insert_text: Some(s),
        kind: Some(CompletionItemKind::VALUE),
        ..CompletionItem::default()
    })
    .collect()
}

fn block_tags() -> Vec<CompletionItem> {
    vec![
        "script".to_string(),
        "style".to_string(),
        "pre".to_string(),
        "figure".to_string(),
    ]
    .into_iter()
    .map(|s| CompletionItem {
        label: s.clone(),
        insert_text: Some(s),
        kind: Some(CompletionItemKind::VALUE),
        ..CompletionItem::default()
    })
    .collect()
}

fn get_styles(line: &str, styles: PathBuf) -> Vec<CompletionItem> {
    match fs::read_dir(styles.as_path()) {
        Ok(paths) => {
            let mut found = vec![CompletionItem {
                label: "Vale".to_string(),
                insert_text: Some("Vale".to_string()),
                kind: Some(CompletionItemKind::VALUE),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some("4 rules".to_string()),
                    ..CompletionItemLabelDetails::default()
                }),
                detail: Some("Style".to_string()),
                ..CompletionItem::default()
            }];

            let dirs: Vec<PathBuf> = paths
                .into_iter()
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap().path())
                .filter(|r| r.is_dir())
                .collect();

            for path in dirs {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let size = fs::read_dir(path.clone()).unwrap().count();
                if name != "Vocab" && name != ".vale-config" && !line.contains(&name) {
                    found.push(CompletionItem {
                        label: name.clone(),
                        insert_text: Some(name.clone()),
                        kind: Some(CompletionItemKind::VALUE),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: path.display().to_string(),
                        })),
                        label_details: Some(CompletionItemLabelDetails {
                            description: Some(format!("{} rules", size)),
                            ..CompletionItemLabelDetails::default()
                        }),
                        detail: Some("Style".to_string()),
                        ..CompletionItem::default()
                    });
                }
            }

            found
        }
        Err(_) => vec![],
    }
}
