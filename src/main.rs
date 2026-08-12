use std::env;

#[derive(Clone, Copy, PartialEq, Eq)]
enum HexSplit {
    None,
    Pairs,
    Quads,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DecSplit {
    None,
    Comma,
    Period,
    Underscore,
}

struct Config {
    show_prefix: bool,
    hex_padding: bool,
    hex_split: HexSplit,
    dec_split: DecSplit,
}

impl Config {
    fn from_env() -> Self {
        Config {
            show_prefix: env_bool("show_prefix", true),
            hex_padding: env_bool("hex_padding", false),
            hex_split: match env::var("hex_splitting").as_deref() {
                Ok("pairs") => HexSplit::Pairs,
                Ok("quads") => HexSplit::Quads,
                _ => HexSplit::None,
            },
            dec_split: match env::var("dec_splitting").as_deref() {
                Ok("comma") => DecSplit::Comma,
                Ok("period") => DecSplit::Period,
                Ok("underscore") => DecSplit::Underscore,
                _ => DecSplit::None,
            },
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|v| matches!(v.as_str(), "1" | "true"))
        .unwrap_or(default)
}

#[derive(Debug, PartialEq, Eq)]
enum Base {
    Hex,
    Dec,
    Oct,
    Bin,
}

#[derive(Debug, PartialEq, Eq)]
struct Number {
    neg: bool,
    mag: u64,
}

fn parse(input: &str) -> Option<(Number, Base)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (digits, radix, base) = if let Some(r) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        (r, 16, Base::Hex)
    } else if let Some(r) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        (r, 2, Base::Bin)
    } else if let Some(r) = s.strip_prefix('b').or_else(|| s.strip_prefix('B')) {
        (r, 2, Base::Bin)
    } else if s.len() > 1 && s.starts_with('0') && s[1..].chars().all(|c| c.is_ascii_digit()) {
        (&s[1..], 8, Base::Oct)
    } else {
        (s, 10, Base::Dec)
    };
    let cleaned: String = digits.chars().filter(|&c| c != '_').collect();
    if cleaned.is_empty() {
        return None;
    }
    let mag = u64::from_str_radix(&cleaned, radix).ok()?;
    if neg && mag > (i64::MAX as u64) + 1 {
        return None;
    }
    Some((
        Number {
            neg: neg && mag != 0,
            mag,
        },
        base,
    ))
}

fn group(s: &str, size: usize, sep: &str) -> String {
    let n = s.len();
    let mut out = String::with_capacity(n + n / size * sep.len());
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (n - i) % size == 0 {
            out.push_str(sep);
        }
        out.push(c);
    }
    out
}

fn format_hex(n: u64, cfg: &Config) -> String {
    let mut s = format!("{:x}", n);
    if cfg.hex_padding {
        let unit = match cfg.hex_split {
            HexSplit::Quads => 4,
            _ => 2,
        };
        let rem = s.len() % unit;
        if rem != 0 {
            s = format!("{}{}", "0".repeat(unit - rem), s);
        }
    }
    let s = match cfg.hex_split {
        HexSplit::None => s,
        HexSplit::Pairs => group(&s, 2, " "),
        HexSplit::Quads => group(&s, 4, " "),
    };
    if cfg.show_prefix {
        format!("0x{s}")
    } else {
        s
    }
}

fn format_dec(n: u64, cfg: &Config) -> String {
    let s = n.to_string();
    match cfg.dec_split {
        DecSplit::None => s,
        DecSplit::Comma => group(&s, 3, ","),
        DecSplit::Period => group(&s, 3, "."),
        DecSplit::Underscore => group(&s, 3, "_"),
    }
}

fn format_oct(n: u64, cfg: &Config) -> String {
    let s = format!("{:o}", n);
    if cfg.show_prefix {
        format!("0{s}")
    } else {
        s
    }
}

fn format_bin(n: u64, cfg: &Config) -> String {
    let raw = format!("{:b}", n);
    let pad = (8 - raw.len() % 8) % 8;
    let padded = format!("{}{}", "0".repeat(pad), raw);
    let grouped = group(&padded, 4, " ");
    if cfg.show_prefix {
        format!("0b{grouped}")
    } else {
        grouped
    }
}

fn twos_complement(mag: u64) -> (u64, u32) {
    for w in [8u32, 16, 32] {
        if mag <= 1u64 << (w - 1) {
            return ((1u64 << w).wrapping_sub(mag), w);
        }
    }
    (0u64.wrapping_sub(mag), 64)
}

fn format_bin_twos(mag: u64, cfg: &Config) -> (String, u32) {
    let (bits, width) = twos_complement(mag);
    let raw = format!("{:0width$b}", bits, width = width as usize);
    let grouped = group(&raw, 4, " ");
    let s = if cfg.show_prefix {
        format!("0b{grouped}")
    } else {
        grouped
    };
    (s, width)
}

fn sign(neg: bool) -> &'static str {
    if neg {
        "-"
    } else {
        ""
    }
}

struct Item {
    title: String,
    subtitle: String,
    valid: bool,
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn to_json(items: &[Item]) -> String {
    let body = items
        .iter()
        .map(|it| {
            format!(
                "{{\"title\":\"{}\",\"subtitle\":\"{}\",\"arg\":\"{}\",\"valid\":{}}}",
                json_escape(&it.title),
                json_escape(&it.subtitle),
                json_escape(&it.title),
                it.valid
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"items\":[{body}]}}")
}

fn main() {
    let cfg = Config::from_env();
    let input = env::args().nth(1).unwrap_or_default();
    let token = input.split_whitespace().next().unwrap_or("");

    let items = if token.is_empty() {
        vec![Item {
            title: "Usage: x <number>".into(),
            subtitle: "e.g. 100 | -100 | 0x64 | 0144 | 0b1100100 | b1100100".into(),
            valid: false,
        }]
    } else {
        match parse(token) {
            Some((n, _)) => {
                let neg = n.neg;
                let mag = n.mag;
                let mut items = vec![
                    Item {
                        title: format!("{}{}", sign(neg), format_hex(mag, &cfg)),
                        subtitle: "Hexadecimal".into(),
                        valid: true,
                    },
                    Item {
                        title: format!("{}{}", sign(neg), format_dec(mag, &cfg)),
                        subtitle: "Decimal".into(),
                        valid: true,
                    },
                    Item {
                        title: format!("{}{}", sign(neg), format_oct(mag, &cfg)),
                        subtitle: "Octal".into(),
                        valid: true,
                    },
                    Item {
                        title: format!("{}{}", sign(neg), format_bin(mag, &cfg)),
                        subtitle: "Binary".into(),
                        valid: true,
                    },
                ];
                if neg {
                    let (twos, width) = format_bin_twos(mag, &cfg);
                    items.push(Item {
                        title: twos,
                        subtitle: format!("Binary (2's complement, {width}-bit)"),
                        valid: true,
                    });
                }
                items
            }
            None => vec![Item {
                title: format!("Invalid number: {token}"),
                subtitle: "Expect decimal, 0x hex, 0 octal, or 0b/b binary (optional - sign)".into(),
                valid: false,
            }],
        }
    };

    println!("{}", to_json(&items));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config {
            show_prefix: true,
            hex_padding: false,
            hex_split: HexSplit::None,
            dec_split: DecSplit::None,
        }
    }

    fn num(neg: bool, mag: u64) -> Number {
        Number { neg, mag }
    }

    #[test]
    fn parse_bases() {
        assert_eq!(parse("100"), Some((num(false, 100), Base::Dec)));
        assert_eq!(parse("0x64"), Some((num(false, 100), Base::Hex)));
        assert_eq!(parse("0X64"), Some((num(false, 100), Base::Hex)));
        assert_eq!(parse("0144"), Some((num(false, 100), Base::Oct)));
        assert_eq!(parse("0b1100100"), Some((num(false, 100), Base::Bin)));
        assert_eq!(parse("b1100100"), Some((num(false, 100), Base::Bin)));
        assert_eq!(parse("0"), Some((num(false, 0), Base::Dec)));
        assert_eq!(parse("1_000_000"), Some((num(false, 1000000), Base::Dec)));
        assert_eq!(parse("089"), None);
        assert_eq!(parse("0x"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("18446744073709551616"), None);
    }

    #[test]
    fn parse_negative() {
        assert_eq!(parse("-100"), Some((num(true, 100), Base::Dec)));
        assert_eq!(parse("-0x64"), Some((num(true, 100), Base::Hex)));
        assert_eq!(parse("-0144"), Some((num(true, 100), Base::Oct)));
        assert_eq!(parse("-0b1100100"), Some((num(true, 100), Base::Bin)));
        assert_eq!(parse("-0"), Some((num(false, 0), Base::Dec)));
        assert_eq!(parse("-"), None);
        assert_eq!(parse("-9223372036854775808"), Some((num(true, 1u64 << 63), Base::Dec)));
        assert_eq!(parse("-9223372036854775809"), None);
        assert_eq!(parse("18446744073709551615"), Some((num(false, u64::MAX), Base::Dec)));
    }

    #[test]
    fn twos_complement_widths() {
        assert_eq!(twos_complement(100), (0b10011100, 8));
        assert_eq!(twos_complement(128), (0b10000000, 8));
        assert_eq!(twos_complement(200), (0xFF38, 16));
        assert_eq!(twos_complement(1u64 << 31), (1u64 << 31, 32));
        assert_eq!(twos_complement((1u64 << 31) + 1), (0xFFFFFFFF7FFFFFFF, 64));
        assert_eq!(twos_complement(1u64 << 63), (1u64 << 63, 64));
    }

    #[test]
    fn bin_twos_format() {
        assert_eq!(format_bin_twos(100, &cfg()), ("0b1001 1100".to_string(), 8));
        assert_eq!(
            format_bin_twos(200, &cfg()),
            ("0b1111 1111 0011 1000".to_string(), 16)
        );
    }

    #[test]
    fn hex_default() {
        assert_eq!(format_hex(100, &cfg()), "0x64");
        assert_eq!(format_hex(4, &cfg()), "0x4");
    }

    #[test]
    fn hex_padding() {
        let mut c = cfg();
        c.hex_padding = true;
        assert_eq!(format_hex(4, &c), "0x04");
        assert_eq!(format_hex(0x12345, &c), "0x012345");
    }

    #[test]
    fn hex_quads_no_pad() {
        let mut c = cfg();
        c.hex_split = HexSplit::Quads;
        assert_eq!(format_hex(0x12345, &c), "0x1 2345");
    }

    #[test]
    fn hex_quads_with_pad() {
        let mut c = cfg();
        c.hex_split = HexSplit::Quads;
        c.hex_padding = true;
        assert_eq!(format_hex(0x12345, &c), "0x0001 2345");
    }

    #[test]
    fn hex_pairs() {
        let mut c = cfg();
        c.hex_split = HexSplit::Pairs;
        assert_eq!(format_hex(0x12345, &c), "0x1 23 45");
        c.hex_padding = true;
        assert_eq!(format_hex(0x12345, &c), "0x01 23 45");
    }

    #[test]
    fn no_prefix() {
        let mut c = cfg();
        c.show_prefix = false;
        assert_eq!(format_hex(100, &c), "64");
        assert_eq!(format_oct(100, &c), "144");
        assert_eq!(format_bin(100, &c), "0110 0100");
    }

    #[test]
    fn dec_splitting() {
        let mut c = cfg();
        assert_eq!(format_dec(1234567, &c), "1234567");
        c.dec_split = DecSplit::Comma;
        assert_eq!(format_dec(1234567, &c), "1,234,567");
        c.dec_split = DecSplit::Period;
        assert_eq!(format_dec(1234567, &c), "1.234.567");
        c.dec_split = DecSplit::Underscore;
        assert_eq!(format_dec(1234567, &c), "1_234_567");
    }

    #[test]
    fn oct_and_bin() {
        assert_eq!(format_oct(100, &cfg()), "0144");
        assert_eq!(format_bin(100, &cfg()), "0b0110 0100");
        assert_eq!(format_bin(0, &cfg()), "0b0000 0000");
        assert_eq!(format_bin(256, &cfg()), "0b0000 0001 0000 0000");
    }

    #[test]
    fn json_output() {
        let items = vec![Item {
            title: "0x64".into(),
            subtitle: "Hexadecimal".into(),
            valid: true,
        }];
        assert_eq!(
            to_json(&items),
            "{\"items\":[{\"title\":\"0x64\",\"subtitle\":\"Hexadecimal\",\"arg\":\"0x64\",\"valid\":true}]}"
        );
    }
}
