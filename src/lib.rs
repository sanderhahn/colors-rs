use std::{fs::File, io::BufWriter, path::Path};

type RGB = [u8; 3];

pub fn rgb(v: u32) -> RGB {
    let r = (v & 0xff0000) >> 16;
    let g = (v & 0xff00) >> 8;
    let b = (v & 0xff) >> 0;
    [r as u8, g as u8, b as u8]
}

fn min(rgb: RGB) -> u8 {
    rgb[0].min(rgb[1]).min(rgb[2])
}

fn max(rgb: RGB) -> u8 {
    rgb[0].max(rgb[1]).max(rgb[2])
}

// hue in the range: 0-3600
// white and black are bytes: 0-1000
type HWB = (u32, u16, u16);

pub fn hue_to_rgb(hue: u32) -> RGB {
    let h = hue / 600;
    let x = (hue % 600 * 255 / 600) as u8;
    let y = 255 - x;
    match h as u8 % 6 {
        0 => [255, x, 0],
        1 => [y, 255, 0],
        2 => [0, 255, x],
        3 => [0, y, 255],
        4 => [x, 0, 255],
        5 => [255, 0, y],
        _ => unreachable!(),
    }
}

pub fn rgb_to_hue(rgb: RGB) -> u16 {
    let c_min = min(rgb);
    let c_max = max(rgb);
    let delta = c_max - c_min;
    if delta == 0 {
        return 0;
    }
    0
}

pub fn gray(value: u16) -> RGB {
    let value = (255 * value as u32 / 1000) as u8;
    [value, value, value]
}

pub fn mix(p: u16, a: RGB, b: RGB) -> RGB {
    let mut out: RGB = RGB::default();
    for i in 0..=2 {
        let start = (a[i] as i32) * 1000;
        let delta = (b[i] as i32 - a[i] as i32) * (p as i32);
        out[i] = ((start + delta) / 1000) as u8;
    }
    out
}

pub fn hwb_to_rgb(hwb: HWB) -> RGB {
    let v = hwb.1 + hwb.2;
    if v >= 1000 {
        let w = 1000 * hwb.1 as u32 / v as u32;
        return gray(w as u16);
    }
    let hue = hwb.0;
    let w = (255 * hwb.1 as u32 / 1000) as u8;
    let b = (255 * hwb.2 as u32 / 1000) as u8;
    let v = 255 - b;

    let h = hue / 600;
    let mut x = (hue % 600 * 1000 / 600) as i32;
    if h & 1 == 1 {
        x = 1000 - x
    }
    let x = w + (x as i32 * (v as i32 - w as i32) / 1000) as u8;
    match h as u8 % 6 {
        0 => [v, x, w],
        1 => [x, v, w],
        2 => [w, v, x],
        3 => [w, x, v],
        4 => [x, w, v],
        5 => [v, w, x],
        _ => unreachable!(),
    }
}

pub fn rgb_to_hwb(rgb: RGB) -> HWB {
    let &[r, g, b] = &rgb;
    let w = min(rgb);
    let v = max(rgb);
    let black = 255 - v;
    let mut hue = 0;
    if v != w {
        dbg!(r == v);
        dbg!(g == v);
        dbg!(b == v);
        let f = if r == v {
            g as i32 - b as i32
        } else if g == v {
            dbg!(b as i32 - r as i32);
            b as i32 - r as i32
        } else {
            r as i32 - g as i32
        } as i32
            * 1000
            / 256;
        dbg!(f);
        let d = (v as i32 - w as i32) * 1000 / 256;
        dbg!(d);
        hue = if r == v {
            0
        } else if g == v {
            2
        } else {
            4
        } as i32
            * 600;
        hue %= 3600;
        dbg!(hue);
        // hue = (i * 600) + (f * 600 / d);
        // hue *= 600;
        hue += 600 * f / d;
        dbg!(hue);
        hue += 3600;
        hue %= 3600;
        dbg!(hue);
        dbg!(rgb);
    }
    (
        hue as u32,
        (w as u32 * 1000 / 255) as u16,
        (black as u32 * 1000 / 255) as u16,
    )
}

pub struct Pixels {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Pixels {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 3) as usize;
        let data = vec![255; size];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn save_image(&self, name: &str) {
        let path = Path::new(name);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.data).unwrap();
    }

    pub fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, rgb: RGB) {
        for x in x..=x + w {
            for y in y..=y + h {
                self.set(x, y, rgb);
            }
        }
    }

    pub fn set(&mut self, x: u32, y: u32, rgb: RGB) {
        let index = ((y * self.width + x) * 3) as usize;
        self.data[index] = rgb[0];
        self.data[index + 1] = rgb[1];
        self.data[index + 2] = rgb[2];
    }
}

pub fn palette(color: HWB) {
    const SCALE: usize = 4;
    const SIZE: u32 = 1 << SCALE;

    const STEPS: usize = 8;

    let width = STEPS << SCALE;
    let height = STEPS << SCALE;
    let mut pixels = Pixels::new(width as u32, height as u32);
    for w in 0..STEPS {
        for b in 0..STEPS {
            let x = (b as u32) << SCALE;
            let y = (w as u32) << SCALE;
            let w = (1000 * w) / (STEPS - 1);
            let b = (1000 * b) / (STEPS - 1);
            let rgb = hwb_to_rgb((color.0, w as u16, b as u16));
            pixels.rect(x, y, SIZE - 1, SIZE - 1, rgb);
        }
    }
    let name = format!("images/palette{}.png", color.0 / 10);
    pixels.save_image(&name);
}

pub fn hue_palette() {
    const SCALE: usize = 4;
    const SIZE: u32 = 1 << SCALE;

    const WIDTH_STEPS: u32 = 360 / 15;
    const HEIGHT_STEPS: u32 = 200 / 20;

    let mut pixels = Pixels::new(
        WIDTH_STEPS << SCALE as u32,
        HEIGHT_STEPS << SCALE as u32,
    );
    for hue in (0..360).step_by(15) {
        for value in (0..200).step_by(20) {
            let x = ((hue / 15) << SCALE) as u32;
            let y = ((value / 20) << SCALE) as u32;
            let b = 100 - (value + 10).min(100);
            let w = (value as i16 - 100).max(0);
            let rgb = hwb_to_rgb((hue * 10 as u32, w as u16 * 10, b as u16 * 10));
            pixels.rect(x, y, SIZE - 1, SIZE - 1, rgb);
        }
    }
    pixels.save_image(&"images/hue_palette.png");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        assert_eq!(rgb_to_hwb(rgb(0xff0000)), (0, 0, 0));
        assert_eq!(rgb_to_hwb(rgb(0xff8000)), (300, 0, 0));
        assert_eq!(rgb_to_hwb(rgb(0x00ff00)), (1200, 0, 0));
        assert_eq!(rgb_to_hwb(rgb(0x0000ff)), (2400, 0, 0));
        assert_eq!(rgb_to_hwb(rgb(0xffff00)), (600, 0, 0));
        assert_eq!(rgb_to_hwb(rgb(0x00ffff)), (1800, 0, 0));
        assert_eq!(rgb_to_hwb(rgb(0xff00ff)), (3000, 0, 0));

        assert_eq!(rgb_to_hwb(rgb(0xcc3333)), (0, 200, 200));
        assert_eq!(rgb_to_hwb(rgb(0x808080)), (0, 500, 500));
    }

    #[test]
    fn test_hue_to_rgb() {
        assert_eq!(hue_to_rgb(0), rgb(0xff0000));
        assert_eq!(hue_to_rgb(600), rgb(0xffff00));
        assert_eq!(hue_to_rgb(1200), rgb(0x00ff00));
        assert_eq!(hue_to_rgb(1800), rgb(0x00ffff));
        assert_eq!(hue_to_rgb(2400), rgb(0x0000ff));
        assert_eq!(hue_to_rgb(3000), rgb(0xff00ff));
    }

    #[test]
    fn test_hwb_block_red() {
        assert_eq!(hwb_to_rgb((0, 0, 0)), rgb(0xff0000));
        assert_eq!(hwb_to_rgb((0, 200, 0)), rgb(0xff3333));
        assert_eq!(hwb_to_rgb((0, 400, 0)), rgb(0xff6666));
        assert_eq!(hwb_to_rgb((0, 600, 0)), rgb(0xff9999));
        assert_eq!(hwb_to_rgb((0, 800, 0)), rgb(0xffcccc));
        assert_eq!(hwb_to_rgb((0, 1000, 0)), rgb(0xffffff));

        assert_eq!(hwb_to_rgb((0, 0, 0)), rgb(0xff0000));
        assert_eq!(hwb_to_rgb((0, 0, 200)), rgb(0xcc0000));
        assert_eq!(hwb_to_rgb((0, 0, 400)), rgb(0x990000));
        assert_eq!(hwb_to_rgb((0, 0, 600)), rgb(0x660000));
        assert_eq!(hwb_to_rgb((0, 0, 800)), rgb(0x330000));
        assert_eq!(hwb_to_rgb((0, 0, 1000)), rgb(0x000000));

        assert_eq!(hwb_to_rgb((0, 0, 1000)), rgb(0x000000));
        assert_eq!(hwb_to_rgb((0, 200, 1000)), rgb(0x2a2a2a));
        assert_eq!(hwb_to_rgb((0, 400, 1000)), rgb(0x484848));
        assert_eq!(hwb_to_rgb((0, 600, 1000)), rgb(0x5f5f5f));
        assert_eq!(hwb_to_rgb((0, 800, 1000)), rgb(0x717171));
        assert_eq!(hwb_to_rgb((0, 1000, 1000)), rgb(0x7f7f7f));

        assert_eq!(hwb_to_rgb((0, 1000, 1000)), rgb(0x7f7f7f));
        assert_eq!(hwb_to_rgb((0, 1000, 800)), rgb(0x8d8d8d));
        assert_eq!(hwb_to_rgb((0, 1000, 600)), rgb(0x9f9f9f));
        assert_eq!(hwb_to_rgb((0, 1000, 400)), rgb(0xb6b6b6));
        assert_eq!(hwb_to_rgb((0, 1000, 200)), rgb(0xd4d4d4));
        assert_eq!(hwb_to_rgb((0, 1000, 0)), rgb(0xffffff));

        assert_eq!(hwb_to_rgb((0, 0, 200)), rgb(0xcc0000));
        assert_eq!(hwb_to_rgb((0, 200, 200)), rgb(0xcc3333));
        assert_eq!(hwb_to_rgb((0, 400, 200)), rgb(0xcc6666));
        assert_eq!(hwb_to_rgb((0, 600, 200)), rgb(0xcc9999));
        assert_eq!(hwb_to_rgb((0, 800, 200)), rgb(0xcccccc));
        assert_eq!(hwb_to_rgb((0, 1000, 200)), rgb(0xd4d4d4));

        assert_eq!(hwb_to_rgb((0, 0, 400)), rgb(0x990000));
        assert_eq!(hwb_to_rgb((0, 200, 400)), rgb(0x993333));
        assert_eq!(hwb_to_rgb((0, 400, 400)), rgb(0x996666));
        assert_eq!(hwb_to_rgb((0, 600, 400)), rgb(0x999999));
        assert_eq!(hwb_to_rgb((0, 800, 400)), rgb(0xa9a9a9));
        assert_eq!(hwb_to_rgb((0, 1000, 400)), rgb(0xb6b6b6));

        assert_eq!(hwb_to_rgb((0, 0, 600)), rgb(0x660000));
        assert_eq!(hwb_to_rgb((0, 200, 600)), rgb(0x663333));
        assert_eq!(hwb_to_rgb((0, 400, 600)), rgb(0x666666));
        assert_eq!(hwb_to_rgb((0, 600, 600)), rgb(0x7f7f7f));
        assert_eq!(hwb_to_rgb((0, 800, 600)), rgb(0x919191));
        assert_eq!(hwb_to_rgb((0, 1000, 600)), rgb(0x9f9f9f));

        assert_eq!(hwb_to_rgb((0, 0, 800)), rgb(0x330000));
        assert_eq!(hwb_to_rgb((0, 200, 800)), rgb(0x333333));
        assert_eq!(hwb_to_rgb((0, 400, 800)), rgb(0x545454));
        assert_eq!(hwb_to_rgb((0, 600, 800)), rgb(0x6d6d6d));
        assert_eq!(hwb_to_rgb((0, 800, 800)), rgb(0x7f7f7f));
        assert_eq!(hwb_to_rgb((0, 1000, 800)), rgb(0x8d8d8d));
    }

    #[test]
    fn test_hwb_block_orange() {
        assert_eq!(hwb_to_rgb((300, 0, 0)), rgb(0xff7f00));
        assert_eq!(hwb_to_rgb((300, 200, 0)), rgb(0xff9933));
        assert_eq!(hwb_to_rgb((300, 400, 0)), rgb(0xffb266));
        assert_eq!(hwb_to_rgb((300, 600, 0)), rgb(0xffcc99));
        assert_eq!(hwb_to_rgb((300, 800, 0)), rgb(0xffe5cc));
        assert_eq!(hwb_to_rgb((300, 1000, 0)), rgb(0xffffff));
    }

    #[test]
    fn test_gray() {
        assert_eq!(gray(500), [127, 127, 127]);
    }

    #[test]
    fn test_mix() {
        assert_eq!(mix(500, [255, 0, 127], [0, 255, 127]), [127, 127, 127]);
    }

    #[test]
    fn test_palettes() {
        for hue in (0..360).step_by(30) {
            let color: HWB = (hue * 10, 0, 0);
            palette(color);
        }
    }

    #[test]
    fn test_hue_palette() {
        hue_palette();
    }
}
