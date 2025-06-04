use anyhow::bail;
use bitvec::{order::Lsb0, vec::BitVec, view::AsBits};
use clap::{Parser, Subcommand, ValueEnum};
use clap_stdin::MaybeStdin;
use phf::{phf_map, phf_set};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(
    after_help = "Examples:\n  uni_stego -m emoji encode -t \"Hello world\"\n  uni_stego -m emoji encode -t \"Secret message\" -c 🔒\n  uni_stego -m emoji decode -t \"👍...\""
)]
struct Cli {
    #[arg(value_enum, short, long)]
    method: Method,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a message into an emoji
    Encode {
        /// Message to encode
        #[arg(short, long)]
        text: MaybeStdin<String>,

        /// Emoji to encode the message into
        #[arg(short, long, default_value = "👍")]
        cover: String,
    },
    /// Decode a message from an emoji
    Decode {
        /// Message to decode
        #[arg(short, long)]
        text: MaybeStdin<String>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum Method {
    Emoji,
    Homoglyph,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode { text, cover } => {
            println!("{}", encode(cli.method, text, cover)?);
        }
        Commands::Decode { text } => {
            println!("{}", decode(cli.method, text)?);
        }
    }

    Ok(())
}

fn encode(method: Method, text: &str, cover: &str) -> anyhow::Result<String> {
    match method {
        Method::Emoji => encode_emoji(text, cover),
        Method::Homoglyph => encode_homoglyph(text, cover),
        #[allow(unreachable_patterns)]
        _ => bail!("Encoding with method {method:?} is not implemented yet."),
    }
}

fn decode(method: Method, text: &str) -> anyhow::Result<String> {
    match method {
        Method::Emoji => decode_emoji(text),
        Method::Homoglyph => decode_homoglyph(text),
        #[allow(unreachable_patterns)]
        _ => bail!("Decoding with method {method:?} is not implemented yet."),
    }
}

fn encode_homoglyph(text: &str, cover: &str) -> anyhow::Result<String> {
    let mut output = String::new();
    let mut bits = text.as_bits::<Lsb0>().iter().peekable();
    let count = text.len() * 8;

    // println!("'{text}' is {} bits long", text.as_bits::<Lsb0>().len());
    // println!("Should be {} bits", text.len() * 8);
    //
    let mut hidden = 0;

    for c in cover.chars() {
        if let Some(bit) = bits.peek() {
            if let Some(new_c) = get_homoglyph(c, **bit) {
                output.push(new_c);
                bits.next();
                hidden += 1;
                continue;
            }
        }
        output.push(c);
    }

    if bits.len() != 0 {
        bail!(
            "Cover text is not long enough to encode message. Capacity: {hidden}. Required: {count}. Remaining: {}",
            bits.len()
        )
    }

    Ok(output)
}

fn decode_homoglyph(text: &str) -> anyhow::Result<String> {
    let mut bits = BitVec::<u8, Lsb0>::new();

    for c in text.chars() {
        if let Some(bit) = get_bit(c) {
            bits.push(bit);
        }
    }
    Ok(String::from_utf8_lossy(bits.as_raw_slice()).to_string())
}

fn get_homoglyph(c: char, bit: bool) -> Option<char> {
    let homoglyph = HOMOGLYPHS.get(&c)?;
    if bit { Some(*homoglyph) } else { Some(c) }
}

fn get_bit(c: char) -> Option<bool> {
    CHARS.contains(&c).then(|| HOMOGLYPHS.get(&c).is_none())
}

static HOMOGLYPHS: phf::Map<char, char> = phf_map! {
    '\u{002d}' => '\u{2010}',
    '\u{003b}' => '\u{037e}',
    '\u{0043}' => '\u{216d}',
    '\u{0044}' => '\u{216e}',
    '\u{004b}' => '\u{212a}',
    '\u{004c}' => '\u{216c}',
    '\u{004d}' => '\u{216f}',
    '\u{0056}' => '\u{2164}',
    '\u{0058}' => '\u{2169}',
    '\u{0063}' => '\u{217d}',
    '\u{0064}' => '\u{217e}',
    '\u{0069}' => '\u{2170}',
    '\u{006a}' => '\u{0458}',
    '\u{006c}' => '\u{217c}',
    '\u{0076}' => '\u{2174}',
    '\u{0078}' => '\u{2179}',
};

static CHARS: phf::Set<char> = phf_set![
    '\u{2010}', '\u{002d}', '\u{037e}', '\u{003b}', '\u{216d}', '\u{0043}', '\u{216e}', '\u{0044}',
    '\u{212a}', '\u{004b}', '\u{216c}', '\u{004c}', '\u{216f}', '\u{004d}', '\u{2164}', '\u{0056}',
    '\u{2169}', '\u{0058}', '\u{217d}', '\u{0063}', '\u{217e}', '\u{0064}', '\u{2170}', '\u{0069}',
    '\u{0458}', '\u{006a}', '\u{217c}', '\u{006c}', '\u{2174}', '\u{0076}', '\u{2179}', '\u{0078}',
];

fn encode_emoji(text: &str, cover: &str) -> anyhow::Result<String> {
    let mut output = String::with_capacity(text.len());
    if cover.chars().count() != 1 {
        println!("{}", cover.len());
        bail!("Cover text must be exactly one character long");
    }
    output.push(cover.chars().next().unwrap());

    for byte in text.as_bytes() {
        output.push(get_char(*byte));
    }

    Ok(output)
}

fn decode_emoji(text: &str) -> anyhow::Result<String> {
    let mut buffer = Vec::with_capacity(text.len());
    for c in text.chars() {
        if let Some(byte) = get_byte(c) {
            buffer.push(byte);
        }
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
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
