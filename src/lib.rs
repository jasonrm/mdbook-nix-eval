use std::fs::File;

use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use tempfile::tempdir;
use serde_json::Value;

pub struct NixEval;

impl NixEval {
    pub fn new() -> NixEval {
        NixEval
    }
}

impl Preprocessor for NixEval {
    fn name(&self) -> &str { "nix-eval" }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|book| {
            if let mdbook::BookItem::Chapter(chapter) = book {
                // TODO: better error handling...
                if let Err(e) = nix_eval(chapter) {
                    eprintln!("nix-eval error: {:?}", e);
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn nix_eval(chapter: &mut Chapter) -> Result<(), Error> {
    use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind, CowStr};

    let chapter_temp_dir = tempdir()?;

    // mini state machine for the current nix-eval tag
    let mut nix: Option<String> = None;
    let events = Parser::new(&chapter.content)
        .filter_map(|event| {
            match &event {
                // a code block for the `nix-eval` language was started
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                    if !(info.ends_with(".nix") || info.to_string() == "nix") {
                        return Some(event);
                    }
                    nix = Some("".to_owned());
                    None
                }
                // a code block for the `nix-eval` language was ended
                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                    let is_file = info.ends_with(".nix");
                    let is_eval = info.to_string() == "nix";
                    if !(is_file || is_eval) {
                        return Some(event);
                    }

                    let nix_file_name = match is_file {
                        true => info.as_ref(),
                        false => "eval.nix",
                    };
                    let nix_file_path = chapter_temp_dir.path().join(nix_file_name);

                    // extract the contents of the diagram
                    let nix_src = nix.take().expect("nix was started");

                    let mut out_file = File::create(nix_file_path.as_path()).expect("nix file created");
                    out_file.write_all(nix_src.as_ref()).expect("wrote temp file");

                    eprintln!("writing temp file: {:?}", nix_file_path.as_path());

                    // evaluate the nix expression
                    use std::process::{Command, Stdio};
                    use std::io::Write;
                    let child = match Command::new("nix-instantiate")
                        .current_dir(chapter_temp_dir.path())
                        .arg("--read-write-mode")
                        .arg("--strict")
                        .arg("--json")
                        .arg("--eval")
                        // .arg("--timeout").arg("30")
                        .arg(nix_file_path.as_path())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn() {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("failed to launch nix-eval, not rendering nix-eval block: {:?}", e);
                            return None;
                        }
                    };

                    let cmd_output = child.wait_with_output().expect("can launch nix-eval");

                    let svg: String = String::from_utf8(cmd_output.stdout).expect("valid utf-8");
                    let mut nix_eval_output = "".to_owned();
                    if !cmd_output.status.success() {
                        nix_eval_output = String::from_utf8(cmd_output.stderr).expect("valid utf-8");
                    } else {
                        let v: Value = serde_json::from_str(svg.as_str()).expect("invalid json");
                        let line = match v {
                            Value::String(s) => {
                                let trimmed = s.trim();
                                if trimmed.contains("\n") {
                                    format!("\"\n{}\n\"", trimmed)
                                } else {
                                    format!("\"{}\"", trimmed)
                                }
                            },
                            Value::Bool(b) => serde_json::to_string_pretty(&b).unwrap(),
                            Value::Null => "null".to_owned(),
                            Value::Number(n) => format!("{}", n),
                            Value::Array(a) => serde_json::to_string_pretty(&a).unwrap(),
                            Value::Object(o) => serde_json::to_string_pretty(&o).unwrap(),
                        };
                        nix_eval_output.push_str(line.as_str())
                    }

                    let input_header = match is_file {
                        true => format!("**{}**\n", info.as_ref()),
                        false => "".to_string(),
                    };

                    let input = format!("\n```nix\n{}\n```\n", nix_src.trim());
                    let output = format!("\n```json\n{}\n```\n", nix_eval_output.trim());

                    nix = None;
                    Some(Event::Text(CowStr::from(format!("\n{}\n<div style='border-left: 2px solid;'>\n{}\n\n{}\n</div>\n\n", input_header, input, output))))
                }
                // intercept text events if we're currently in the code block state
                Event::Text(txt) => {
                    if let Some(nix) = nix.as_mut() {
                        nix.push_str(&txt);
                        None
                    } else {
                        Some(event)
                    }
                }
                // don't touch other events
                _ => Some(event),
            }
        });

    let mut buf = String::with_capacity(chapter.content.len());
    pulldown_cmark_to_cmark::cmark(events, &mut buf, None).expect("can re-render cmark");
    chapter.content = buf;

    Ok(())
}

