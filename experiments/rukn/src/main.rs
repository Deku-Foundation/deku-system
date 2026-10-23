use std::env;
use std::fs;
use std::process::ExitCode;

use rukn::{check, run, Diagnostic, Phase, Span};

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn entry() -> Result<(), Diagnostic> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();

    if command == "--version" || command == "-V" {
        println!("rukn {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if command == "--help" || command == "-h" || command.is_empty() {
        print_help();
        return if command.is_empty() {
            Err(Diagnostic::new(
                Phase::Io,
                Span::new(1, 1),
                "لم تحدد أمرًا؛ استخدم «rukn --help»",
            ))
        } else {
            Ok(())
        };
    }

    if command != "run" && command != "check" {
        return Err(Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            format!("أمر غير معروف: {command}"),
        ));
    }

    let path = args.next().ok_or_else(|| {
        Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            format!("الأمر «{command}» يتطلب مسار ملف"),
        )
    })?;

    if args.next().is_some() {
        return Err(Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            "الإصدار الأول يقبل ملفًا واحدًا لكل أمر",
        ));
    }

    let source = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            format!("تعذر قراءة الملف «{path}»: {error}"),
        )
    })?;

    if command == "check" {
        check(&source)?;
        println!("التحليل النحوي والتحقق الدلالي سليمان؛ لم يتم تنفيذ الشيفرة.");
    } else {
        for line in run(&source)? {
            println!("{line}");
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        "Rukn — مفسر لغة ركن التجريبي\n\nالاستخدام:\n  rukn run <file.rukn>    تحليل الملف وتنفيذه\n  rukn check <file.rukn>  التحقق دون تنفيذ\n  rukn --version          عرض الإصدار\n  rukn --help             عرض هذه المساعدة"
    );
}
