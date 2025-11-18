// In the sprawl of identical terminals, your mark matters. Every run gets a face, 
// random font-face and color bleeding different each time. They can copy the code,
// but they can't copy the signature. That's yours.

use figlet_rs::FIGfont;
use colored::*;

pub fn print_title() {
    let (_font_name, font_bytes) = crate::fonts::random_font();
    if let Ok(decompressed) = crate::fonts::decompress_font(font_bytes) {
        let font_str = String::from_utf8_lossy(&decompressed);
        match FIGfont::from_content(&font_str) {
            Ok(font) => {
                if let Some(figure) = font.convert("speedrun") {
                    print_gradient_text(&figure.to_string());
                    println!("speedrun v{}\n", env!("CARGO_PKG_VERSION"));
                    return;
                }
            }
            Err(_) => {}
        }
    }
    
    // Fallback to standard font if something goes wrong
    if let Ok(standard_font) = FIGfont::standard() {
        if let Some(figure) = standard_font.convert("speedrun") {
            print_gradient_text(&figure.to_string());
            println!("speedrun v{}\n", env!("CARGO_PKG_VERSION"));
        }
    }
}

fn print_gradient_text(text: &str) {
    // Randomly select a gradient preset
    use colorgrad::Gradient;
    use rand::prelude::IndexedRandom;
    
    let gradients: Vec<Box<dyn Gradient>> = vec![
        Box::new(colorgrad::preset::rainbow()),
        Box::new(colorgrad::preset::sinebow()),
        Box::new(colorgrad::preset::turbo()),
        Box::new(colorgrad::preset::viridis()),
        Box::new(colorgrad::preset::plasma()),
        Box::new(colorgrad::preset::magma()),
        Box::new(colorgrad::preset::inferno()),
        Box::new(colorgrad::preset::warm()),
        Box::new(colorgrad::preset::cool()),
        Box::new(colorgrad::preset::cubehelix_default()),
        Box::new(colorgrad::preset::spectral()),
        Box::new(colorgrad::preset::rd_yl_gn()),
        Box::new(colorgrad::preset::rd_yl_bu()),
        Box::new(colorgrad::preset::pu_or()),
        Box::new(colorgrad::preset::br_bg()),
    ];
    
    let mut rng = rand::rng();
    let gradient = gradients.choose(&mut rng).unwrap();
    
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return;
    }
    
    // Find the maximum line width
    let max_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    
    if max_width == 0 {
        println!("{}", text);
        return;
    }
    
    // Print each line with gradient applied horizontally
    for line in lines {
        for (i, ch) in line.chars().enumerate() {
            let t = i as f32 / max_width as f32;
            let color = gradient.at(t);
            let rgba = color.to_rgba8();
            print!("{}", ch.to_string().truecolor(rgba[0], rgba[1], rgba[2]));
        }
        println!();
    }
}
