use std::{env, process::exit};

enum Mode {
    Encode,
    Decode,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        usage(Some("ERROR: no arguments given."));
    }

    println!("{:?}", args);

    let mode = match args.get(1).map(|s| s.as_str()) {
        Some("-e") | Some("--encode") => Mode::Encode,
        Some("-d") | Some("--decode") => Mode::Decode,
        Some("-h") | Some("--help") => {
            usage(None);
        }
        Some(_) => {
            usage("ERROR: Invalid mode must be one of: -e/--encode, -d/--deocde");
        }
        None => unreachable!(),
    };

    match mode {
        Mode::Encode => {
            let Some(message) = args.get(2) else {
                usage("ERROR: Must provide message.");
            };

            /* let message = args.get(2).cloned().unwrap_or_else(|| {
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();
                buffer
            }); */

            let emoji = match args.get(3) {
                Some(string) => {
                    if string.len() != 1 {
                        usage("ERROR: Emoji must be exactly one character.");
                    }
                    string.chars().next().expect("String should be length one.")
                }
                None => '👍',
            };

            println!("{}", encode(message, emoji));
        }
        Mode::Decode => {
            let Some(message) = args.get(2) else {
                usage("ERROR: Must provide message.");
            };

            println!("{}", decode(message));
        }
    }
}

fn encode(message: &str, emoji: char) -> String {
    let mut output = String::with_capacity(message.len());
    output.push(emoji);

    for byte in message.as_bytes() {
        output.push(get_char(*byte));
    }

    output
}

fn decode(message: &str) -> String {
    let mut buffer = Vec::with_capacity(message.len());
    for c in message.chars() {
        if let Some(byte) = get_byte(c) {
            buffer.push(byte);
        }
    }

    String::from_utf8_lossy(&buffer).to_string()
}

// const CHAR_MAP: OnceCell<Vec<char>> = OnceCell::new();

const VARIATION_SELECTOR_START: u32 = 0xFE00;
const VARIATION_SELECTOR_END: u32 = 0xFE0F;
const VARIATION_SELECTOR_SUPPLEMENT_START: u32 = 0xE0100;
const VARIATION_SELECTOR_SUPPLEMENT_END: u32 = 0xE01EF;

fn get_char(byte: u8) -> char {
    let code_point = if byte < 16 {
        VARIATION_SELECTOR_START + byte as u32
    } else {
        VARIATION_SELECTOR_SUPPLEMENT_START + byte as u32
    };
    unsafe { char::from_u32_unchecked(code_point) }
}

fn get_byte(c: char) -> Option<u8> {
    let code_point = c as u32;
    if (VARIATION_SELECTOR_START..=VARIATION_SELECTOR_END).contains(&code_point) {
        Some((code_point - VARIATION_SELECTOR_START) as u8)
    } else if (VARIATION_SELECTOR_SUPPLEMENT_START..=VARIATION_SELECTOR_SUPPLEMENT_END)
        .contains(&code_point)
    {
        Some((code_point - VARIATION_SELECTOR_SUPPLEMENT_START) as u8)
    } else {
        None
    }
}

fn usage<'a>(message: impl Into<Option<&'a str>>) -> ! {
    let message = message.into();
    if let Some(message) = message {
        println!("{message}");
    }
    println!("USAGE: emoji_stego [mode] [<message>] [<emoji>]");
    println!("\tmode:");
    println!("\t -e, --encode \t Encode a message into an emoji");
    println!("\t -d, --decode \t Decode a message from an emoji");
    println!("\t emoji \t Emoji to encode the message into. Optional when encoding (default: 👍)");
    println!("\t message \t Message to encode into an emoji.");

    exit(if message.is_some() { 1 } else { 0 });
}
