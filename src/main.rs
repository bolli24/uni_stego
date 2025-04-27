use clap::{Parser, Subcommand};
use clap_stdin::MaybeStdin;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(
    after_help = "Examples:\n  emoji_stego encode -m \"Hello world\"\n  emoji_stego encode -m \"Secret message\" -e 🔒\n  emoji_stego decode -m \"👍...\""
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a message into an emoji
    Encode {
        /// Message to encode
        #[arg(short, long)]
        message: MaybeStdin<String>,

        /// Emoji to encode the message into
        #[arg(short, long, default_value = "👍")]
        emoji: char,
    },
    /// Decode a message from an emoji
    Decode {
        /// Message to decode
        #[arg(short, long)]
        message: MaybeStdin<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode { message, emoji } => {
            println!("{}", encode(message, *emoji));
        }
        Commands::Decode { message } => {
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
        VARIATION_SELECTOR_SUPPLEMENT_START + byte as u32 - 16
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
        Some((code_point - VARIATION_SELECTOR_SUPPLEMENT_START + 16) as u8)
    } else {
        None
    }
}
