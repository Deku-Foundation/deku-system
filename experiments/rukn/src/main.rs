use std::env;
use std::fs;
use std::io::Read;
use std::process::ExitCode;

use rukn::{Diagnostic, MAX_SOURCE_BYTES, Phase, Span, check, run};

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

    let file = fs::File::open(&path).map_err(|error| {
        Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            format!("تعذر فتح الملف «{path}»: {error}"),
        )
    })?;
    let mut bytes = Vec::new();
    file.take((MAX_SOURCE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            Diagnostic::new(
                Phase::Io,
                Span::new(1, 1),
                format!("تعذرت قراءة الملف «{path}»: {error}"),
            )
        })?;
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            format!("تجاوز الملف حد الحجم البالغ {MAX_SOURCE_BYTES} بايت"),
        ));
    }
    let source = String::from_utf8(bytes).map_err(|error| {
        Diagnostic::new(
            Phase::Io,
            Span::new(1, 1),
            format!("الملف «{path}» ليس UTF-8 صالحًا: {error}"),
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
