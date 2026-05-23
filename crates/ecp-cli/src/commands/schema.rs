use clap::{Args, Subcommand};
use ecp_core::EcpError;
use serde::Serialize;

#[derive(Args, Clone)]
pub struct SchemaArgs {
    #[command(subcommand)]
    pub command: SchemaCommands,
}

#[derive(Subcommand, Clone)]
pub enum SchemaCommands {
    /// Per-language BlindSpot emitter inventory.
    ///
    /// Distinguishes "no blind spot in this diff" from "ecp doesn't detect
    /// this dispatch pattern in this language yet" — exactly the
    /// LLM-context signal Constraint 5 of the cross-lang spec requires.
    Blindspots(BlindspotsArgs),
}

#[derive(Args, Clone)]
pub struct BlindspotsArgs {
    /// Output format. Default `json`; `text` is a human-readable table.
    #[arg(long, default_value = "json")]
    pub format: String,
}

/// Detection status for a per-lang capability. Append-only enum (string
/// JSON discriminator — no rkyv discriminant to worry about).
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Status {
    Implemented,
    None,
}

#[derive(Serialize)]
struct LangEntry {
    name: &'static str,
    blindspot_emitter: Status,
    indirect_dispatch: Status,
    blind_kinds: &'static [&'static str],
}

#[derive(Serialize)]
struct BlindspotsReport {
    languages: &'static [LangEntry],
}

/// Per-lang inventory. Hardcoded because the per-parser `BLIND_SPEC`
/// tables are `pub(crate)` and reflecting them at runtime is overkill for
/// a stable spec. Kept beside `parse_file` impl in each parser; whoever
/// adds a new lang updates both places.
///
/// Order: same as in `~/.ecp/index/{repo}/lang_counts.json` (alphabetical
/// for stable diff output).
const LANGUAGES: &[LangEntry] = &[
    LangEntry {
        name: "c",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::Implemented,
        blind_kinds: &["c-dlsym"],
    },
    LangEntry {
        name: "cpp",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::Implemented,
        blind_kinds: &["cpp-dlsym"],
    },
    LangEntry {
        name: "c_sharp",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["cs-activator-create-instance", "cs-method-invoke"],
    },
    LangEntry {
        name: "dart",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["dart-function-apply", "dart-mirrors-import"],
    },
    LangEntry {
        name: "go",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["go-reflect-method-by-name", "go-plugin-open"],
    },
    LangEntry {
        name: "java",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["java-class-forname", "java-method-invoke"],
    },
    LangEntry {
        name: "javascript",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::Implemented,
        blind_kinds: &[
            "js-eval",
            "js-function-ctor",
            "js-dynamic-import",
            "js-dynamic-require",
        ],
    },
    LangEntry {
        name: "kotlin",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["kt-class-forname", "kt-method-invoke"],
    },
    LangEntry {
        name: "php",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["php-eval", "php-call-user-func", "php-variable-call"],
    },
    LangEntry {
        name: "python",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::Implemented,
        blind_kinds: &[
            "python-eval",
            "python-exec",
            "python-compile",
            "python-dynamic-import",
            "python-builtin-import",
            "python-cross-getattr",
        ],
    },
    LangEntry {
        name: "ruby",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["rb-eval", "rb-instance-eval", "rb-send"],
    },
    LangEntry {
        name: "rust",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::Implemented,
        blind_kinds: &["rs-transmute-fn", "rs-libloading-get"],
    },
    LangEntry {
        name: "swift",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::None,
        blind_kinds: &["swift-nsclass-from-string", "swift-perform-selector"],
    },
    LangEntry {
        name: "typescript",
        blindspot_emitter: Status::Implemented,
        indirect_dispatch: Status::Implemented,
        blind_kinds: &[
            "ts-eval",
            "ts-function-ctor",
            "ts-dynamic-import",
            "ts-dynamic-require",
        ],
    },
];

pub fn run(args: SchemaArgs) -> Result<(), EcpError> {
    match args.command {
        SchemaCommands::Blindspots(a) => blindspots(a),
    }
}

fn blindspots(args: BlindspotsArgs) -> Result<(), EcpError> {
    let report = BlindspotsReport {
        languages: LANGUAGES,
    };
    match args.format.as_str() {
        "json" => {
            let s = serde_json::to_string_pretty(&report)
                .map_err(|e| EcpError::Serialization(format!("blindspots report: {e}")))?;
            println!("{}", s);
        }
        "text" => print_text(&report),
        other => {
            return Err(EcpError::InvalidArgument(format!(
                "unknown --format `{}`; expected `json` or `text`",
                other
            )));
        }
    }
    Ok(())
}

fn print_text(report: &BlindspotsReport) {
    println!("lang             emitter   indirect  kinds");
    println!("--------------------------------------------------");
    for lang in report.languages {
        let emitter = match lang.blindspot_emitter {
            Status::Implemented => "yes",
            Status::None => "no",
        };
        let indirect = match lang.indirect_dispatch {
            Status::Implemented => "yes",
            Status::None => "no",
        };
        println!(
            "{:<16} {:<9} {:<9} {}",
            lang.name,
            emitter,
            indirect,
            lang.blind_kinds.join(", ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_14_mainstream_languages_listed() {
        // CLAUDE.md parser change rule: 14 mainstream langs must have
        // BlindSpot coverage. This test pins the schema-cmd output to the
        // same set so dropping a lang here triggers a CI failure.
        let names: Vec<&str> = LANGUAGES.iter().map(|l| l.name).collect();
        for expected in &[
            "typescript",
            "javascript",
            "python",
            "java",
            "kotlin",
            "c_sharp",
            "go",
            "rust",
            "php",
            "ruby",
            "swift",
            "c",
            "cpp",
            "dart",
        ] {
            assert!(
                names.contains(expected),
                "missing language `{}` in schema-cmd inventory",
                expected
            );
        }
    }

    #[test]
    fn every_language_has_at_least_one_blind_kind() {
        for lang in LANGUAGES {
            assert!(
                !lang.blind_kinds.is_empty(),
                "language `{}` has empty blind_kinds — either remove it or add an emitter",
                lang.name
            );
        }
    }

    #[test]
    fn blind_kinds_are_unique_globally() {
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for lang in LANGUAGES {
            for kind in lang.blind_kinds {
                assert!(
                    seen.insert(kind),
                    "duplicate blind kind `{}` across languages",
                    kind
                );
            }
        }
    }
}
