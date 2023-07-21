use std::{env, str::FromStr};

use ropey::Rope;
use tower_lsp::lsp_types::*;

use crate::pkg;
use crate::styles;
use crate::vale;

pub(crate) fn make_title(action: String, matched: String, fix: String) -> String {
    match action.as_str() {
        "remove" => format!("Remove ‘{}’", matched),
        _ => format!("Replace with ‘{}’", fix),
    }
}

pub(crate) fn vale_arch() -> String {
    let platform = match env::consts::OS {
        "windows" => "Windows",
        "macos" => "macOS",
        _ => "Linux",
    };
    let arch = match env::consts::ARCH {
        "x86_64" => "64-bit",
        "arm" => "arm64",
        "aarch64" => "arm64",
        _ => "386",
    };
    format!("{}_{}", platform, arch)
}

pub(crate) fn position_to_range(p: Position, rope: &Rope) -> Option<Range> {
    let line = p.line as usize;
    let index = p.character as usize;

    let context = rope.line(line);
    let extent = context.chars().count() - 1;

    let mut start = index;
    while start > 0 && !context.char(start - 1).is_whitespace() {
        start -= 1;
    }

    let mut end = index;
    while end < extent && !context.char(end + 1).is_whitespace() {
        end += 1;
    }

    if start == end {
        return None;
    } else if end > index {
        // TODO: Why is this necessary?
        //
        // FIXME:
        //
        // BasedOnStyles = Vale
        //                   ^
        end += 1;
    }

    Some(Range::new(
        Position::new(line as u32, start as u32),
        Position::new(line as u32, end as u32),
    ))
}

pub(crate) fn range_to_token(r: Range, rope: &Rope) -> String {
    let start = r.start.character as usize;
    let end = r.end.character as usize;

    let context = rope.line(r.start.line as usize);
    let token = context.slice(start..end).as_str().unwrap_or("");

    token.to_string()
}

pub(crate) fn alert_to_range(alert: vale::ValeAlert) -> Range {
    Range {
        start: Position {
            line: alert.line as u32 - 1,
            character: alert.span.0 as u32 - 1,
        },
        end: Position {
            line: alert.line as u32 - 1,
            character: alert.span.1 as u32,
        },
    }
}

pub(crate) fn severity_to_level(severity: String) -> DiagnosticSeverity {
    match severity.as_str() {
        "error" => DiagnosticSeverity::ERROR,
        "warning" => DiagnosticSeverity::WARNING,
        "suggestion" => DiagnosticSeverity::INFORMATION,
        _ => DiagnosticSeverity::HINT,
    }
}

pub(crate) fn entry_to_completion(v: styles::PathEntry) -> CompletionItem {
    CompletionItem {
        label: v.name.clone(),
        insert_text: Some(v.name.clone()),
        kind: Some(CompletionItemKind::VALUE),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: v.path.display().to_string(),
        })),
        detail: Some(v.kind.to_string()),
        ..CompletionItem::default()
    }
}

pub(crate) fn pkg_to_completion(pkg: pkg::Package) -> CompletionItem {
    CompletionItem {
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
    }
}

pub(crate) fn alert_to_diagnostic(alert: &vale::ValeAlert) -> Diagnostic {
    let mut d = Diagnostic {
        range: alert_to_range(alert.clone()),
        severity: Some(severity_to_level(alert.severity.clone())),
        code: Some(NumberOrString::String(alert.check.clone())),
        source: Some("vale-ls".to_string()),
        message: alert.message.clone(),
        related_information: None,
        code_description: None,
        tags: None,
        data: Some(serde_json::to_value(alert).unwrap()),
    };

    if alert.link != "" {
        let uri = Url::from_str(&alert.link);
        if uri.is_ok() {
            d.code_description = Some(CodeDescription {
                href: Some(uri.unwrap()).unwrap(),
            });
        }
    }

    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch() {
        let arch = vale_arch();
        match env::consts::OS {
            "windows" => assert_eq!(arch, "Windows_64-bit"),
            "macos" => assert!(arch == "macOS_64-bit" || arch == "macOS_arm64"),
            _ => assert_eq!(arch, "Linux_64-bit"),
        }
    }
}
