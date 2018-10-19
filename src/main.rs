extern crate reqwest;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate clap;
extern crate image;
extern crate serde_json;

use clap::{App, Arg};
use image::{FilterType, GenericImageView};

#[derive(Debug, Deserialize)]
struct Xkcd {
    alt: String,
    day: String,
    img: String,
    link: String,
    month: String,
    news: String,
    num: u32,
    safe_title: String,
    title: String,
    transcript: String,
    year: String,
}

fn main() -> Result<(), Box<std::error::Error>> {
    let matches = App::new("xkcli")
        .version("0.1")
        .author("Andy L. <andy@wire.com>")
        .about("xkcd viewer for a more civilized age")
        .arg(
            Arg::with_name("index")
                .help("Index of xkcd to show (defaults to the most recent one)")
                .default_value(""),
        ).arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .help("Maximum character of the image")
                .default_value("100"),
        ).get_matches();

    let char_width = value_t!(matches, "width", u32).unwrap_or(100);
    let xkcd_index = matches.value_of("index").unwrap();
    let mut img_buf: Vec<u8> = vec![];

    let json: Xkcd =
        reqwest::get(&["http://xkcd.com/", xkcd_index, "/info.0.json"].concat())?.json()?;
    let mut img_req = reqwest::get(&json.img)?;
    img_req.copy_to(&mut img_buf)?;
    let original_img = image::load_from_memory(&img_buf)?;
    let (_, height) = original_img.dimensions();
    let img = original_img.resize(char_width * 2, height, FilterType::Lanczos3);
    let (width, height) = img.dimensions();
    let (width, height) = (width as usize, height as usize);

    let bool_height = (height as f32 / 4.).ceil() as usize * 4;
    let bool_width = (width as f32 / 2.).ceil() as usize * 2;

    let mut values = vec![vec![0_f32; height]; width];
    let mut bool_values = vec![vec![false; bool_height]; bool_width];

    for x in 0..width {
        for y in 0..height {
            let [r, g, b, _] = img.get_pixel(x as u32, y as u32).data;
            let (r, g, b) = (r as f32 / 255_f32, g as f32 / 255_f32, b as f32 / 255_f32);
            let val = values[x][y] + 0.299 * r + 0.587 * g + 0.114 * b;
            let norm_val = if val < 0.5 { 0_f32 } else { 1_f32 };
            bool_values[x][y] = if norm_val == 0_f32 { true } else { false };
            let error = val - norm_val;
            if x < (width - 1) {
                values[x + 1][y] += error * (7.0 / 16.0);
            }
            if y < (height - 1) {
                values[x][y + 1] += error * (5.0 / 16.0);
                if x > 0 {
                    values[x - 1][y + 1] += error * (3.0 / 16.0)
                }
                if x < (width - 1) {
                    values[x + 1][y + 1] += error * (1.0 / 16.0);
                }
            }
        }
    }

    let mut output = "".to_string();

    for y in 0..(bool_height / 4) {
        for x in 0..(bool_width / 2) {
            let (x, y) = (x * 2, y * 4);
            output.push_str(&get_braille(
                bool_values[x][y],
                bool_values[x + 1][y],
                bool_values[x][y + 1],
                bool_values[x + 1][y + 1],
                bool_values[x][y + 2],
                bool_values[x + 1][y + 2],
                bool_values[x][y + 3],
                bool_values[x + 1][y + 3],
            ));
        }
        output.push_str("\n");
    }

    println!(
        "#{} ({}-{}-{}): {}\n",
        json.num, json.year, json.month, json.day, json.title
    );
    println!("{}", output);
    Ok(())
}

fn get_braille(
    d1_1: bool,
    d1_2: bool,
    d2_1: bool,
    d2_2: bool,
    d3_1: bool,
    d3_2: bool,
    d4_1: bool,
    d4_2: bool,
) -> String {
    let num = 0x2800_u16
        | d1_1 as u16
        | ((d2_1 as u16) << 1)
        | ((d3_1 as u16) << 2)
        | ((d1_2 as u16) << 3)
        | ((d2_2 as u16) << 4)
        | ((d3_2 as u16) << 5)
        | ((d4_1 as u16) << 6)
        | ((d4_2 as u16) << 7);
    String::from_utf16(&[num]).unwrap()
}
