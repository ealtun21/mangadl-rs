#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Encoding {
    Unicode,
    Ascii,
}

// Get the encoding from the command line arguments
pub fn get_encoding(args: &Vec<String>) -> Encoding {
    if args.iter().any(|arg| arg == "--ascii") {
        Encoding::Ascii
    } else {
        Encoding::Unicode
    }
}

pub fn display_help(args: &Vec<String>) -> bool {
    if args.iter().any(|args| args == "--help") {
        println!("Usage: mangadl-rs [OPTIONS]");
        println!("\nOptions:");
        println!("  --ascii\t\tUse ascii characters instead of unicode");
        println!("  --help\t\tDisplay this help message");
        return true;
    }
    false
}
