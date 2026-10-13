//! `s3d` 命令行（M1：compile）。

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("compile") => {
            eprintln!("compile：尚未实现（M1）");
            std::process::exit(2);
        }
        _ => {
            eprintln!("用法：s3d compile <文件.s3d> [--json]");
            std::process::exit(2);
        }
    }
}
