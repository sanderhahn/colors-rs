use std::fmt::Display;

#[derive(Copy, Clone)]
pub struct RGB {
    red: u8,
    green: u8,
    blue: u8,
}

impl RGB {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn from_hsl(hue: f32, saturation: f32, luminance: f32) -> RGB {
        let hsl = &HSL::new(hue, saturation, luminance);
        hsl.into()
    }

    pub fn primary_colors(step: usize) -> Vec<RGB> {
        (0..360)
            .step_by(step)
            .map(|hue| RGB::from_hsl(hue as f32, 1.0, 1.0))
            .collect()
    }

    pub fn gray(intensity: f32) -> RGB {
        RGB::from_hsl(0.0, 0.0, intensity)
    }
}

impl Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

#[derive(Copy, Clone)]
pub struct HSL {
    hue: f32,
    saturation: f32,
    luminance: f32,
}

impl HSL {
    pub fn new(hue: f32, saturation: f32, luminance: f32) -> Self {
        Self {
            hue,
            saturation,
            luminance,
        }
    }

    pub fn black() -> HSL {
        Self {
            hue: 0.0,
            saturation: 0.0,
            luminance: 0.0,
        }
    }

    pub fn primary_colors(step: usize) -> Vec<HSL> {
        (0..360)
            .step_by(step)
            .map(|hue| HSL::new(hue as f32, 1.0, 1.0))
            .collect()
    }
}

impl Into<RGB> for &HSL {
    fn into(self) -> RGB {
        fn hex(v: f32) -> u8 {
            (v * 255f32) as u8
        }

        // https://en.wikipedia.org/wiki/HSL_and_HSV
        let c = self.luminance * self.saturation;
        let h = self.hue / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let rgb = match h as u8 % 6 {
            0 => (c, x, 0.),
            1 => (x, c, 0.),
            2 => (0., c, x),
            3 => (0., x, c),
            4 => (x, 0., c),
            5 => (c, 0., x),
            _ => unreachable!(),
        };
        let m = self.luminance - c;

        RGB {
            red: hex(rgb.0 + m),
            green: hex(rgb.1 + m),
            blue: hex(rgb.2 + m),
        }
    }
}

impl Display for HSL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rgb: RGB = self.into();
        write!(f, "{}", rgb)
    }
}

fn main() {
    let colors = HSL::primary_colors(15);
    for saturation in (0..=4).rev() {
        println!(r#"<div style="display: display: table;">"#);
        for intensity in 0..=15 {
            println!(r#"<div style="display: display: table-row;">"#);
            for &color in &colors {
                println!(
                    r#"<div style="display: table-cell; background-color: {}; width: 16px; height: 16px;"></div>"#,
                    HSL {
                        saturation: saturation as f32 / 4.0,
                        luminance: intensity as f32 / 15.0,
                        ..color
                    }
                );
            }
            println!("</div>");
        }
        println!("</div>");
        println!("<br>");
    }
    println!("</div>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as FmtWrite;

    #[test]
    fn test_format() {
        assert_eq!("#ff0000", format!("{}", RGB::new(0xff, 0, 0)));
    }

    #[test]
    fn test_hsl() {
        let mut colors = String::new();
        for color in RGB::primary_colors(30) {
            write!(colors, "{}\n", color).unwrap();
        }
        assert_eq!(colors, "#ff0000\n#ff7f00\n#ffff00\n#7fff00\n#00ff00\n#00ff7f\n#00ffff\n#007fff\n#0000ff\n#7f00ff\n#ff00ff\n#ff007f\n");
    }

    #[test]
    fn test_grays() {
        let mut colors = String::new();
        for gray in 0..=4 {
            write!(colors, "{}\n", RGB::gray(gray as f32 / 4.0)).unwrap();
        }
        assert_eq!(colors, "#000000\n#3f3f3f\n#7f7f7f\n#bfbfbf\n#ffffff\n");
    }
}
