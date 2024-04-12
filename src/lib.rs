#[cfg(feature = "create_phrases")]
pub mod generate_phrase {
    use image::{ImageBuffer, Rgba};
    use imageproc::drawing::draw_text_mut;
    use reqwest::get;
    use rusttype::{Font, Scale};
    use std::fs::read;

    pub async fn create_image(
        avatar: &str,
        content: &str,
        name: &str,
        font_path: &str,
        italic_font_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Descarga la imagen del avatar
        let resp = get(avatar).await?;
        let bytes = resp.bytes().await?;
        let mut avatar_img = image::load_from_memory(&bytes)?.to_rgba8();
        // Redimensiona el avatar a un tamaño más pequeño
        avatar_img =
            image::imageops::resize(&avatar_img, 150, 150, image::imageops::FilterType::Nearest);

        // Crea una nueva imagen con un tamaño específico y fondo negro
        let mut img: ImageBuffer<Rgba<u8>, _> =
            ImageBuffer::from_pixel(700, 182, Rgba([0u8, 0u8, 0u8, 255u8]));

        // Dibuja el avatar en la imagen en la posición deseada (un poco más a la derecha y cerca del centro)
        let avatar_x = 100; // 100 pixels to the right
        let avatar_y = (img.height() / 2) - (avatar_img.height() / 2);
        image::imageops::overlay(&mut img, &avatar_img, avatar_x, i64::from(avatar_y));

        // Carga la fuente para el texto del autor
        //let font = Vec::from(include_bytes!(font_path) as &[u8]);
        let font = read(font_path)?;
        let font = Font::try_from_vec(font).unwrap(); // SAFETY: Siempre hay una fuente, en caso de fallo comprobar la ruta

        // Carga la fuente Italic para el texto citado
        //let italic_font = Vec::from(include_bytes!(italic_font_path) as &[u8]);
        let italic_font = read(italic_font_path)?;
        let italic_font = Font::try_from_vec(italic_font).unwrap(); // SAFETY: Siempre hay una fuente, en caso de fallo comprobar la ruta

        // Dibuja el texto en la imagen en la posición deseada (más a la derecha y a una altura similar a la del avatar)
        let height = 30.0; // Increase the text size
        let int_height = 30; // Increase the text size
        let scale = Scale {
            x: height,
            y: height,
        }; // Make sure x and y are the same to avoid squishing
        let text_x = img.width() - 350; // 350 pixels from the right edge
        let text_x: i32 = text_x.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32
        let text_y = avatar_y + 25; // Este valor mueve el texto del contenido del mensaje hacia arriba o abajo dependiendo del valor ("+" para abajo, "-" para arriba")

        // Define current_height before drawing the content
        let current_height = text_y;
        let mut current_height = current_height.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32

        // Divide el contenido en palabras y dibuja cada línea por separado
        let max_width = 300.0; // Maximum width of the text
        let words = content.split_whitespace();
        let mut line = String::new();
        for word in words {
            let glyphs = italic_font.glyphs_for(word.chars());
            let word_width = glyphs
                .map(|g| g.scaled(scale).h_metrics().advance_width)
                .sum::<f32>();
            if word_width > max_width {
                // If the word is too long, split it into multiple lines
                let chars = word.chars().collect::<Vec<char>>();
                let mut sub_word = String::new();
                for ch in chars {
                    let new_sub_word = format!("{sub_word}{ch}");
                    let sub_word_width = italic_font
                        .glyphs_for(new_sub_word.chars())
                        .map(|g| g.scaled(scale).h_metrics().advance_width)
                        .sum::<f32>();
                    if sub_word_width > max_width {
                        // Draw the line and start a new one
                        draw_text_mut(
                            &mut img,
                            Rgba([255u8, 255u8, 255u8, 255u8]),
                            text_x,
                            current_height,
                            scale,
                            &italic_font,
                            &sub_word,
                        );
                        current_height += int_height; // Move to the next line
                        sub_word = ch.to_string();
                    } else {
                        sub_word = new_sub_word;
                    }
                }
                line = sub_word;
            } else {
                let new_line = format!("{line} {word}");
                let line_width = italic_font
                    .glyphs_for(new_line.chars())
                    .map(|g| g.scaled(scale).h_metrics().advance_width)
                    .sum::<f32>();
                if line_width > max_width {
                    // Draw the line and start a new one
                    draw_text_mut(
                        &mut img,
                        Rgba([255u8, 255u8, 255u8, 255u8]),
                        text_x,
                        current_height,
                        scale,
                        &italic_font,
                        &line,
                    );
                    current_height += int_height; // Move to the next line
                    line = word.to_string();
                } else {
                    line = new_line;
                }
            }
        }
        // Draw the last line
        draw_text_mut(
            &mut img,
            Rgba([255u8, 255u8, 255u8, 255u8]),
            text_x,
            current_height,
            scale,
            &italic_font,
            &line,
        );

        // Dibuja el nombre del autor en la imagen en una posición independiente del contenido del mensaje
        let name_y = img.height() - 50; // 100 pixels from the bottom edge
        let name_y = name_y.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32

        let name_x = img.width() - 300; // 300 pixels from the right edge
        let name_x = name_x.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32
        draw_text_mut(
            &mut img,
            Rgba([255u8, 255u8, 255u8, 255u8]),
            name_x,
            name_y,
            scale,
            &font,
            name,
        );

        // Guarda la imagen
        let path = format!("/tmp/{content}_phrase.png");
        img.save(&path)?;

        Ok(path)
    }
}

#[cfg(feature = "create_welcome")]
pub mod create_welcome {
    use image::imageops::{overlay, resize};
    use image::{imageops, GenericImage, GenericImageView, ImageBuffer, ImageError, Pixel, Rgba};

    fn create_round_avatar<I: GenericImageView<Pixel = Rgba<u8>>>(
        avatar: &I,
        target_size: u32,
    ) -> impl GenericImage<Pixel = Rgba<u8>> {
        // Redimensiona el avatar al tamaño deseado antes de aplicar la máscara circular
        let avatar = resize(avatar, target_size, target_size, imageops::Lanczos3);
        let (width, height) = avatar.dimensions();
        let radius = width as f32 / 2.0;
        let mut mask = ImageBuffer::new(width, height);
        let center = (width as f32 / 2.0, height as f32 / 2.0);

        for (x, y, pixel) in mask.enumerate_pixels_mut() {
            let dx = x as f32 - center.0 + 0.5; // +0.5 para centrar el pixel
            let dy = y as f32 - center.1 + 0.5;
            if dx.powi(2) + dy.powi(2) <= radius.powi(2) {
                *pixel = Rgba([255, 255, 255, 255]);
            } else {
                *pixel = Rgba([0, 0, 0, 0]);
            }
        }

        // Aplica la máscara al avatar redimensionado
        ImageBuffer::from_fn(width, height, |x, y| {
            let mask_pixel = mask.get_pixel(x, y).0[3];
            let avatar_pixel = avatar.get_pixel(x, y);
            if mask_pixel > 0 {
                *avatar_pixel
            } else {
                avatar_pixel.map_with_alpha(|f| f, |_| 0)
            }
        })
    }

    pub fn combine_images<I: GenericImage<Pixel = Rgba<u8>>>(
        background: &mut I,
        avatar: &I,
        x: u32,
        y: u32,
        target_size: u32,
    ) -> Result<(), ImageError> {
        let round_avatar = create_round_avatar(avatar, target_size);

        let adjusted_x = if x >= 10 { x - 10 } else { 0 };
        let adjusted_y = if y >= 10 { y - 10 } else { 0 };

        for (ax, ay, pixel) in round_avatar.pixels() {
            let bx = adjusted_x + ax;
            let by = adjusted_y + ay;
            if bx < background.width() && by < background.height() {
                let alpha = background.get_pixel(bx, by).0[3];

                if alpha < 127 {
                    background.put_pixel(bx, by, pixel);
                }
            }
        }

        // Calcula la escala y redimensiona si es necesario
        let (avatar_width, avatar_height) = round_avatar.dimensions();
        if adjusted_x + avatar_width > background.width()
            || adjusted_y + avatar_height > background.height()
        {
            // Si el avatar es demasiado grande, calcula la nueva escala
            let scale_x = (background.width() - adjusted_x) as f64 / avatar_width as f64;
            let scale_y = (background.height() - adjusted_y) as f64 / avatar_height as f64;
            let scale = scale_x.min(scale_y);

            // Calcula las nuevas dimensiones
            let new_width = (avatar_width as f64 * scale) as u32;
            let new_height = (avatar_height as f64 * scale) as u32;

            // Redimensiona el avatar
            let resized_avatar = resize(
                &round_avatar,
                new_width,
                new_height,
                imageops::FilterType::Lanczos3,
            );
            overlay(
                background,
                &resized_avatar,
                adjusted_x as i64,
                adjusted_y as i64,
            );
        } else {
            // Si el avatar ya cabe, colócalo directamente
            overlay(
                background,
                &round_avatar,
                adjusted_x as i64,
                adjusted_y as i64,
            );
        }

        Ok(())
    }
}