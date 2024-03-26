use std::io::Write;
use std::io::BufRead;

static THEMES: &'static str = "/Users/mschmutz/.config/alacritty/themes";
static CONFIG: &'static str = "/Users/mschmutz/.config/alacritty/alacritty.toml";
static BASE: &'static str = "~/.config/alacritty/themes/";

fn get_themes(theme_dir: String) -> Vec<std::fs::DirEntry> {
    let mut themes = vec![];
    match std::fs::read_dir(theme_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => themes.push(entry),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    };

    themes
}

fn select_theme(themes: &Vec<std::fs::DirEntry>) -> usize {
    let space = 40;
    for (idx, el) in themes.iter().enumerate() {
        let prefix = space - idx.to_string().len();
        print!("{}: {:<space$}", idx, el.file_name().into_string().unwrap(), space = prefix);
        if (idx + 1) % 5 == 0 {
            println!("");
        }
    }
    let mut input = String::new();
    let mut theme: usize = usize::MAX;

    println!("");
    print!("Please choose a theme > ");
    std::io::stdout().flush().expect("Flush didn't work?");
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim();
            let result: Result<usize, _> = trimmed.parse();
            theme = result.unwrap();
        },
        Err(e) => eprintln!("Error: {}", e)
    };

    theme
}

fn read_config() -> Vec<String> {
    let file = std::fs::File::open(CONFIG).expect("File does not exist");
    let buf = std::io::BufReader::new(file);
    let lines: Vec<String> = buf.lines()
        .map(|l| l.expect("Couldn't parse line"))
        .collect();

    lines
}

fn apply_theme(theme: &std::fs::DirEntry) -> () {
    let new_theme = BASE.to_string() + &theme.file_name().into_string().unwrap();
    let mut lines = read_config();
    for line in &mut lines {
        if line.contains("import") {
            *line = format!("import = [\"{new_theme}\"]");
        }
    }

    let mut file = std::fs::File::create(CONFIG).expect("Couldn't create file");
    for line in lines {
        file.write_all(line.as_bytes()).expect("Couldn't write line");
        file.write_all(b"\n").expect("couldn't write newline");
    }
}

fn get_color(theme: &std::fs::DirEntry) -> Result<(u8, u8, u8), &str> {
    let file = std::fs::File::open(theme.path()).expect("Could not open the theme file");
    let buf = std::io::BufReader::new(file);
    let lines: Vec<String> = buf.lines()
        .map(|l| l.expect("Couldn't parse line"))
        .collect();

    let mut color_line: String = String::new();
    for line in lines {
        if line.starts_with("#") {
            continue;
        }
        if line.contains("background") {
            color_line = line;
            break;
        }
    }

    let parts: Vec<&str> = color_line.as_str()
        .split('=')
        .map(|part| part.trim())
        .collect();
    let color = parts[1];

    let r = u8::from_str_radix(&color[2..4], 16);
    let g = u8::from_str_radix(&color[4..6], 16);
    let b = u8::from_str_radix(&color[6..8], 16);

    match(r, g, b) {
        (Ok(r), Ok(g), Ok(b)) => Ok((r, g, b)),
        _ => Err("Failed to parse hex color")
    }
}

fn calc_luminance(r: u8, g: u8, b: u8) -> f64 {
    // 0.8 is taking into account the alpha of alacritty
    // TODO: update to grab opacity from toml too
    0.8 * (0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b))
}

fn calc_contrast_color(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    const LIGHT_LUMIN: f64 = 64.0;
    const DARK_LUMIN: f64 = 128.0;
    let adjusted: f64;
    let luminance = calc_luminance(r, g, b);

    // js tinycolor uses 128 as threshold
    if luminance >= 128.0 {
        adjusted = LIGHT_LUMIN / luminance;
    } else {
        adjusted = DARK_LUMIN / luminance;
    };
    let new_r = (r as f64 * adjusted).min(255.0).round() as u8;
    let new_g = (g as f64 * adjusted).min(255.0).round() as u8;
    let new_b = (b as f64 * adjusted).min(255.0).round() as u8;

    (new_r, new_g, new_b)
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let themes: Vec<std::fs::DirEntry> = get_themes(THEMES.to_string());
    let idx = select_theme(&themes);
    apply_theme(&themes[idx]);
    let bg_color: (u8, u8, u8) = get_color(&themes[idx]).expect("Error returning color tuple");
    let contrasted: (u8, u8, u8) = calc_contrast_color(bg_color.0, bg_color.1, bg_color.2);
    // println!("{}", format!("#{:02X}{:02X}{:02X}", bg_color.0, bg_color.1, bg_color.2));
    // println!("{}", format!("#{:02X}{:02X}{:02X}", contrasted.0, contrasted.1, contrasted.2));
}
