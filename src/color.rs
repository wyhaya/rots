use bright::Colorful;
use rand::Rng;
use std::f32::consts::PI;

#[derive(Debug, Copy, Clone)]
struct Option {
    // Seed of the rainbow, use the same for the same pattern
    seed: f32,
    // Spread of the rainbow
    spread: f32,
    // Frequency of the rainbow colors
    freq: f32,
}

impl Option {
    fn new() -> Self {
        Self {
            seed: (rand::thread_rng().gen::<f32>() * 1000.).round().max(1.),
            spread: 8.,
            freq: 0.3,
        }
    }
}

fn rainbow(freq: f32, i: f32) -> (u8, u8, u8) {
    let red = ((freq * i + 0.).sin() * 127. + 128.).round();

    let green = ((freq * i + 2. * PI / 3.).sin() * 127. + 128.).round();

    let blue = (((freq) * i + 4. * PI / 3.).sin() * 127. + 128.).round();

    (red as u8, green as u8, blue as u8)
}

pub fn print(text: String) {
    let mut option = Option::new();
    for item in text.split('\n') {
        for (i, ch) in item.chars().enumerate() {
            let (r, g, b) = rainbow(option.freq, option.seed + i as f32 / option.spread);
            print!("{}", ch.to_string().rgb(r, g, b));
        }
        option.seed += 1_f32;
        println!();
    }
}
