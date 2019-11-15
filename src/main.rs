use image::{DynamicImage, GenericImageView, Pixel};
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "png to 2bpp cli", about = "convert image files to .2bpp")]
struct Opt {
    /// path to file
    #[structopt(parse(from_os_str))]
    image_path: PathBuf,

    /// output file name
    #[structopt(short = "o", long = "output")]
    output_path: Option<PathBuf>,

    /// palette.json defaults to Aseprites gameboy palette
    #[structopt(short = "p", long = "pallete")]
    pallete: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
enum Intensity {
    Lightest, // 9bbc0f
    Light,    // 8bac0f
    Dark,     // 306230
    Darkest,  // 0f380f
}

struct Pallete {
    lightest: [u8; 3],
    light: [u8; 3],
    dark: [u8; 3],
    darkest: [u8; 3],
}

impl Default for Pallete {
    fn default() -> Self {
        Pallete {
            lightest: [0x9b, 0xbc, 0x0f],
            light: [0x8b, 0xac, 0x0f],
            dark: [0x30, 0x62, 0x30],
            darkest: [0x0f, 0x38, 0x0f],
        }
    }
}

#[derive(Debug)]
struct PixelData {
    sprites: Vec<SpritePixelData>,
}

impl PixelData {
    fn init() -> Self {
        let sprites: Vec<SpritePixelData> = Vec::new();
        PixelData { sprites }
    }
}

#[derive(Debug)]
struct SpritePixelData {
    data: [[Intensity; 8]; 8],
}

impl SpritePixelData {
    fn init() -> Self {
        let data = [[Intensity::Lightest; 8]; 8];
        SpritePixelData { data }
    }
}

struct Converter {
    raw_image: DynamicImage,
    sprite_dimensions: (usize, usize),
    pixel_data: PixelData,
    converted: bool,
}

impl Converter {
    fn init(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let raw_dimensions = image::image_dimensions(path)?;

        if raw_dimensions.0 % 8 != 0 && raw_dimensions.1 % 8 != 0 {
            panic!("dimensions incompatible");
        }

        let sprite_dimensions = (
            (raw_dimensions.0 / 8) as usize,
            (raw_dimensions.1 / 8) as usize,
        );

        let pixel_data = PixelData::init();
        let raw_image = image::open(path)?;

        Ok(Converter {
            pixel_data,
            raw_image,
            sprite_dimensions,
            converted: false,
        })
    }

    fn convert(&mut self, pallete: Option<Pallete>) {
        let pallete = match pallete {
            Some(pallete) => pallete,
            None => Default::default(),
        };

        let (lightest, light, dark, darkest) = (
            pallete.lightest,
            pallete.light,
            pallete.dark,
            pallete.darkest,
        );

        for x in 0..self.sprite_dimensions.0 {
            for y in 0..self.sprite_dimensions.1 {
                let mut sprite = SpritePixelData::init();

                for i in 0..8 {
                    for j in 0..8 {
                        let pixel = self
                            .raw_image
                            .get_pixel((x * 8 + i) as u32, (y * 8 + j) as u32)
                            .to_rgb();

                        let pixel = [pixel[0] as u8, pixel[1] as u8, pixel[2] as u8];

                        let intensity = match pixel {
                            _ if pixel == lightest => Intensity::Lightest,
                            _ if pixel == light => Intensity::Light,
                            _ if pixel == dark => Intensity::Dark,
                            _ if pixel == darkest => Intensity::Darkest,
                            _ => panic!("color not in pallete"),
                        };

                        sprite.data[i][j] = intensity;
                    }
                }

                self.pixel_data.sprites.push(sprite);
            }
        }
        self.converted = true;
    }

    fn output(&self) -> Vec<u8> {
        if !self.converted {
            panic!("image has not been converted!");
        }

        let sprites = &self.pixel_data.sprites;
        let mut out: Vec<u8> = Vec::new();

        for i in 0..sprites.len() {
            for j in 0..8 {
                let mut bytes: (u8, u8) = (0, 0);
                for k in 0..8 {
                    let pixel = sprites[i].data[j][k];
                    let bit = k as u8;

                    match pixel {
                        Intensity::Lightest => {}
                        Intensity::Light => {
                            bytes.0 |= 1 << bit;
                        }
                        Intensity::Dark => {
                            bytes.1 |= 1 << bit;
                        }
                        Intensity::Darkest => {
                            bytes.0 |= 1 << bit;
                            bytes.1 |= 1 << bit;
                        }
                    }
                }
                out.push(bytes.0);
                out.push(bytes.1);
            }
        }

        out
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut opt = Opt::from_args();
    let mut conv = Converter::init(&opt.image_path)?;
    conv.convert(None);
    let out = conv.output();
    std::fs::write(
        match opt.output_path {
            Some(path) => path,
            None => {
                opt.image_path.set_extension("2bpp");
                opt.image_path
            }
        },
        out,
    )?;

    Ok(())
}
