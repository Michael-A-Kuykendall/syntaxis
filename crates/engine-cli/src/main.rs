use conllu::{export, import_str};
use english_rules::{evaluation::evaluate, pipeline::analyze, rulepack::RulePack};
use parser_core::ids::TokenId;
use parser_core::support::SourceRef;
use std::env;
use std::fs;
use std::process;

fn main() {
    if let Err(error) = run() {
        eprintln!("syntaxis: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut conllu_input = None;
    let mut conllu_output = false;
    let mut validate = false;
    let mut digest = false;
    let mut retract = None;
    let mut evaluation = false;
    let mut text = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--conllu-in" => {
                conllu_input = Some(args.next().ok_or("--conllu-in requires a path")?);
            }
            "--conllu-out" => conllu_output = true,
            "--validate" => validate = true,
            "--digest" => digest = true,
            "--retract-token" => {
                let value = args.next().ok_or("--retract-token requires an integer")?;
                retract = Some(value.parse::<u32>().map_err(|_| "invalid token id")?);
            }
            "--evaluation" => evaluation = true,
            value if value.starts_with('-') => return Err(format!("unknown option `{value}`")),
            value => text.push(value.to_string()),
        }
    }

    let pack = RulePack::builtin().map_err(|error| error.to_string())?;
    if evaluation {
        let report = evaluate(&pack);
        println!(
            "cases={} clean={} errors={} tp={} fp={} fn={} precision_micros={} recall_micros={} deterministic={}",
            report.total,
            report.clean,
            report.errors,
            report.true_positive,
            report.false_positive,
            report.false_negative,
            report.precision_micros(),
            report.recall_micros(),
            report.deterministic
        );
        return Ok(());
    }
    let mut analysis = if let Some(path) = conllu_input {
        let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
        import_str(&source, &pack.id).map_err(|error| error.to_string())?
    } else {
        if text.is_empty() {
            return Err("provide text or --conllu-in PATH".to_string());
        }
        analyze(&text.join(" "), &pack)
    };

    if let Some(token) = retract {
        analysis.retract(&SourceRef::TokenAnalysis(TokenId(token)));
    }
    if validate {
        for issue in analysis.validate() {
            println!("{issue}");
        }
    }
    if digest {
        println!("{}", analysis.digest());
    }
    if conllu_output {
        print!("{}", export(&analysis));
    } else if !validate && !digest {
        print!("{}", analysis.to_canonical_json());
    }
    Ok(())
}

fn print_help() {
    println!(
        "Syntaxis — deterministic English structural analysis\n\n\
Usage:\n  syntaxis [OPTIONS] TEXT...\n  syntaxis --conllu-in FILE --conllu-out\n\n\
Options:\n  --conllu-in FILE       import strict CoNLL-U instead of analyzing text\n  --conllu-out           emit canonical CoNLL-U\n  --validate             print structural validation issues\n  --digest               print the byte-stable analysis digest\n  --retract-token ID     retract one token analysis before output\n  --evaluation           run the frozen 500-case structural gate\n  -h, --help             show this help"
    );
}
