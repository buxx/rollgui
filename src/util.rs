use crate::error::RollingError;
use crate::level::Level;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use std::{fs, io};

pub const BLOCK_GEO: &str = "GEO";
pub const BLOCK_LEGEND: &str = "LEGEND";

#[derive(Debug)]
pub struct Blinker<T> {
    pub items: HashMap<T, Instant>,
}

impl<T> Blinker<T>
where
    T: std::cmp::Eq + std::hash::Hash,
{
    pub fn visible(&mut self, blink_ms: i32, key: T) -> bool {
        if let Some(instant) = self.items.get(&key) {
            let elapsed = instant.elapsed().as_millis();
            if elapsed < blink_ms as u128 {
                return false;
            } else if elapsed <= (blink_ms * 2) as u128 {
                return true;
            }
        }

        self.items.insert(key, Instant::now());
        false
    }
}

#[derive(Debug)]
pub struct Ticker {
    time: u128, // ms
    instant: Instant,
}

impl Ticker {
    pub fn new(time: u128) -> Self {
        Self {
            time,
            instant: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        if self.instant.elapsed().as_millis() >= self.time {
            self.instant = now;
            return true;
        }
        false
    }
}

pub fn extract_block_from_source(block_name: &str, source: &str) -> Result<String, RollingError> {
    let mut block_found = false;
    let mut block_lines: Vec<&str> = Vec::new();

    for line in source.lines() {
        if line.starts_with("::") {
            // TODO BS 2019-04-03: there is strip method ?
            let line_block_name = line.replace("::", "").replace("\n", "").replace(" ", "");
            if line_block_name == block_name {
                block_found = true;
            } else if block_found {
                return Ok(block_lines.join("\n"));
            }
        } else if block_found {
            block_lines.push(line);
        }
    }

    if block_found {
        return Ok(block_lines.join("\n"));
    }
    Err(RollingError::new(format!(
        "Block \"{}\" not found",
        block_name
    )))
}

pub fn get_file_content(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = fs::File::open(file_path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

pub fn last_char_is(searched_char: char, chars: &Vec<Vec<char>>) -> bool {
    if chars.is_empty() || chars.last().unwrap().is_empty() {
        return false;
    }

    let last_line: &Vec<char> = chars.last().unwrap();
    let inverted_last_line: Vec<&char> = last_line.iter().rev().collect();

    return inverted_last_line[0] == &searched_char;
}

pub fn top_chars_contains(searched_char: char, chars: &Vec<Vec<char>>) -> bool {
    // Consider chars lines length minimum of 3 chars

    if chars.len() < 2 {
        return false;
    }

    let inverted_lines: Vec<&Vec<char>> = chars.iter().rev().collect();
    let previous_line_len = inverted_lines[1].len();
    let ref_char_position = inverted_lines[0].len();

    let mut test_positions: Vec<usize> = Vec::new();

    if ref_char_position == 0 {
        test_positions.push(0);
        test_positions.push(1);
    } else if ref_char_position == previous_line_len - 1 {
        test_positions.push(ref_char_position - 1);
        test_positions.push(ref_char_position);
    } else if ref_char_position == previous_line_len {
        test_positions.push(ref_char_position - 1);
    } else {
        test_positions.push(ref_char_position - 1);
        test_positions.push(ref_char_position);
        test_positions.push(ref_char_position + 1);
    }

    for test_position in test_positions.into_iter() {
        if inverted_lines[1][test_position] == searched_char {
            return true;
        }
    }

    false
}

pub fn longest_line(text: &str) -> Option<&str> {
    let mut max_length = 0;
    let mut longest_line: Option<&str> = None;

    for line in text.lines() {
        let contents = line.trim_end();
        let line_length = contents.len();
        if line_length > max_length {
            max_length = line_length;
            longest_line = Some(contents);
        }
    }

    longest_line
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_block_from_source_ok_one_block() {
        let result =
            extract_block_from_source("BLOCK_NAME", &String::from("::BLOCK_NAME\nline1\nline2"));
        assert_eq!(String::from("line1\nline2"), result.unwrap())
    }

    #[test]
    fn extract_block_from_source_ok_second_block() {
        let result = extract_block_from_source(
            "BLOCK_NAME",
            &String::from("::BLOCKA\nlinea\n::BLOCK_NAME\nline1\nline2"),
        );
        assert_eq!(String::from("line1\nline2"), result.unwrap())
    }

    #[test]
    fn extract_block_from_source_ok_not_last_block() {
        let result = extract_block_from_source(
            "BLOCKA",
            &String::from("::BLOCKA\nlinea\n::BLOCK_NAME\nline1\nline2"),
        );
        assert_eq!(String::from("linea"), result.unwrap())
    }

    #[test]
    #[should_panic]
    fn extract_block_from_source_err_no_block() {
        extract_block_from_source(
            "BLOCK_NAME_UNKNOWN",
            &String::from("::BLOCK_NAME\nline1\nline2"),
        )
        .unwrap();
    }

    #[test]
    fn last_char_is_ok() {
        let chars = vec![vec![]];
        assert!(!last_char_is('a', &chars));

        let chars = vec![vec!['a']];
        assert!(last_char_is('a', &chars));

        let chars = vec![vec!['a', 'b']];
        assert!(!last_char_is('a', &chars));

        let chars = vec![vec!['a', 'b', 'c']];
        assert!(!last_char_is('a', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec![]];
        assert!(!last_char_is('a', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec!['a']];
        assert!(last_char_is('a', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec!['a', 'b']];
        assert!(!last_char_is('a', &chars));
    }

    #[test]
    fn top_chars_contains_ok() {
        let chars = vec![vec![]];
        assert!(!top_chars_contains('a', &chars));

        let chars = vec![vec!['a', 'b', 'c']];
        assert!(!top_chars_contains('a', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec![]];
        assert!(top_chars_contains('a', &chars));
        assert!(top_chars_contains('b', &chars));
        assert!(!top_chars_contains('c', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec!['x']];
        assert!(top_chars_contains('a', &chars));
        assert!(top_chars_contains('b', &chars));
        assert!(top_chars_contains('c', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec!['x', 'y']];
        assert!(!top_chars_contains('a', &chars));
        assert!(top_chars_contains('b', &chars));
        assert!(top_chars_contains('c', &chars));

        let chars = vec![vec!['a', 'b', 'c'], vec!['x', 'y', 'z']];
        assert!(!top_chars_contains('a', &chars));
        assert!(!top_chars_contains('b', &chars));
        assert!(top_chars_contains('c', &chars));
    }
}

pub fn bool_to_str(bool_value: bool) -> &'static str {
    if bool_value {
        return "Oui";
    }
    return "Non";
}

pub fn overflow(text: &str, width: i32) -> Vec<String> {
    let mut lines: Vec<String> = vec![String::new()];

    for word in text.split(" ").collect::<Vec<&str>>() {
        let mut last_line = lines.last_mut().unwrap();
        if last_line.len() + word.len() > width as usize {
            lines.push(String::new());
            last_line = lines.last_mut().unwrap();
        }

        if last_line.len() != 0 {
            last_line.push_str(" ");
        }

        last_line.push_str(word);
    }

    // remove empty line produced by oversize words
    let lines = lines
        .into_iter()
        .filter(|s| s.len() != 0)
        .collect::<Vec<_>>();

    lines
}

#[derive(Debug)]
pub enum CornerEnum {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

const NORTH_WEST_BORDER_CONF: &str = "000\n001\n011";
const LEFT_NORTH_BORDER_CONF: &str = "000\n000\n011";
const STD_NORTH_BORDER_CONF: &str = "000\n000\n111";
const RIGHT_NORTH_BORDER_CONF: &str = "000\n000\n110";
const NORTH_EST_BORDER_CONF: &str = "000\n100\n110";
const TOP_EST_BORDER_CONF: &str = "000\n100\n100";
const STD_EST_BORDER_CONF: &str = "100\n100\n100";
const BOTTOM_EST_BORDER_CONF: &str = "100\n100\n000";
const SOUTH_EST_BORDER_CONF: &str = "110\n100\n000";
const RIGHT_SOUTH_BORDER_CONF: &str = "110\n000\n000";
const STD_SOUTH_BORDER_CONF: &str = "111\n000\n000";
const LEFT_SOUTH_BORDER_CONF: &str = "011\n000\n000";
const SOUTH_WEST_BORDER_CONF: &str = "011\n001\n000";
const BOTTOM_WEST_BORDER_CONF: &str = "001\n001\n000";
const STD_WEST_BORDER_CONF: &str = "001\n001\n001";
const TOP_WEST_BORDER_CONF: &str = "000\n001\n001";

fn get_bin_char_for_border(level: &Level, row_i: i16, col_i: i16) -> char {
    if &level.tile_id(row_i, col_i) == "NOTHING" {
        return '0';
    }
    '1'
}

pub fn get_corner(level: &Level, row_i: i16, col_i: i16) -> Option<CornerEnum> {
    let mut representation: String = String::new();

    representation.push(get_bin_char_for_border(level, row_i - 1, col_i - 1));
    representation.push(get_bin_char_for_border(level, row_i - 1, col_i));
    representation.push(get_bin_char_for_border(level, row_i - 1, col_i + 1));
    representation.push('\n');
    representation.push(get_bin_char_for_border(level, row_i, col_i - 1));
    representation.push(get_bin_char_for_border(level, row_i, col_i));
    representation.push(get_bin_char_for_border(level, row_i, col_i + 1));
    representation.push('\n');
    representation.push(get_bin_char_for_border(level, row_i + 1, col_i - 1));
    representation.push(get_bin_char_for_border(level, row_i + 1, col_i));
    representation.push(get_bin_char_for_border(level, row_i + 1, col_i + 1));

    let representation = representation.as_str();

    if representation == NORTH_WEST_BORDER_CONF {
        Some(CornerEnum::TopLeft)
    } else if representation == LEFT_NORTH_BORDER_CONF
        || representation == STD_NORTH_BORDER_CONF
        || representation == RIGHT_NORTH_BORDER_CONF
    {
        Some(CornerEnum::Top)
    } else if representation == NORTH_EST_BORDER_CONF {
        Some(CornerEnum::TopRight)
    } else if representation == TOP_EST_BORDER_CONF
        || representation == STD_EST_BORDER_CONF
        || representation == BOTTOM_EST_BORDER_CONF
    {
        Some(CornerEnum::Right)
    } else if representation == SOUTH_EST_BORDER_CONF {
        Some(CornerEnum::BottomRight)
    } else if representation == RIGHT_SOUTH_BORDER_CONF
        || representation == STD_SOUTH_BORDER_CONF
        || representation == LEFT_SOUTH_BORDER_CONF
    {
        Some(CornerEnum::Bottom)
    } else if representation == SOUTH_WEST_BORDER_CONF {
        Some(CornerEnum::BottomLeft)
    } else if representation == BOTTOM_WEST_BORDER_CONF
        || representation == STD_WEST_BORDER_CONF
        || representation == TOP_WEST_BORDER_CONF
    {
        Some(CornerEnum::Left)
    } else {
        None
    }
}

pub fn str_version_to_tuple(version: &str) -> (u8, u8, u8) {
    let split: Vec<&str> = version.split(".").collect::<Vec<&str>>();
    let major = split.get(0).unwrap().parse::<u8>().unwrap();
    let minor = split.get(1).unwrap().parse::<u8>().unwrap();
    let correction = split.get(2).unwrap().parse::<u8>().unwrap();
    (major, minor, correction)
}

pub fn is_compatible_versions(server_version: (u8, u8, u8), client_version: (u8, u8, u8)) -> bool {
    println!("{:?} {:?}", server_version, client_version);
    let (server_major, server_minor, _) = server_version;
    let (client_major, client_minor, _) = client_version;

    if server_major != client_major {
        return false;
    }

    if server_minor != client_minor {
        return false;
    }

    true
}

pub fn unzip_to(zip_file: fs::File, to_folder: &Path) {
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => to_folder.join(path.to_owned()),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
}

pub fn get_last_compatible_version(server_version: (u8, u8, u8)) -> (u8, u8, u8) {
    let (server_major, server_minor, _) = server_version;
    let client = reqwest::blocking::Client::new();
    let index_response = client
        .get("http://rolling.bux.fr/release/index")
        .send()
        .unwrap();
    let indexes: String = index_response.text().unwrap();

    let mut last_compatible_version: (u8, u8, u8) = (0, 0, 0);

    for version_as_str in indexes.lines() {
        let version = str_version_to_tuple(version_as_str);
        let (major, minor, _) = version;
        if server_major == major && server_minor == minor {
            last_compatible_version = version;
        }
    }

    last_compatible_version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overflow() {
        assert_eq!(
            overflow("I'm a pingoo with an apple", 15),
            vec!["I'm a pingoo".to_string(), "with an apple".to_string()],
        );
        assert_eq!(
            overflow("I'm a pingoo with an apple", 32),
            vec!["I'm a pingoo with an apple".to_string()],
        );
    }
}

pub fn get_tile_position_for_xy(tile_width: i16, tile_height: i16, x: i16, y: i16) -> (i16, i16) {
    (
        (y + tile_width / 2) / tile_width,
        (x + tile_height / 2) / tile_height,
    )
}

pub fn sleep_if_required(target_frame_ms: u64, last_tick: &SystemTime) {
    let last_tick_duration = last_tick.elapsed().unwrap_or(Duration::from_millis(0));
    let target_duration = Duration::from_millis(target_frame_ms);
    if last_tick_duration < target_duration {
        std::thread::sleep(target_duration - last_tick_duration);
    }
}

pub fn rand_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect()
}
