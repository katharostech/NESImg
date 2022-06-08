use egui::color::linear_f32_from_gamma_u8;
use image::Rgb;
use once_cell::sync::Lazy;

// Uncomment if we need clipboard support later
// pub(crate) static CLIPBOARD: Lazy<Mutex<arboard::Clipboard>> =
//     Lazy::new(|| Mutex::new(arboard::Clipboard::new().expect("Access clipboard")));

// TODO: Use egui::Color32 instead of Rgb<u8>
/// NES color pallet
pub static NES_PALLET: Lazy<[Rgb<u8>; 64]> = Lazy::new(|| {
    [
        Rgb([84, 84, 84]),
        Rgb([0, 30, 116]),
        Rgb([8, 16, 144]),
        Rgb([48, 0, 136]),
        Rgb([68, 0, 100]),
        Rgb([92, 0, 48]),
        Rgb([84, 4, 0]),
        Rgb([60, 24, 0]),
        Rgb([32, 42, 0]),
        Rgb([8, 58, 0]),
        Rgb([0, 64, 0]),
        Rgb([0, 60, 0]),
        Rgb([0, 50, 60]),
        Rgb([0, 0, 0]),
        Rgb([0, 0, 0]),
        Rgb([0, 0, 0]),
        Rgb([152, 150, 152]),
        Rgb([8, 76, 196]),
        Rgb([48, 50, 236]),
        Rgb([92, 30, 228]),
        Rgb([136, 20, 176]),
        Rgb([160, 20, 100]),
        Rgb([152, 34, 32]),
        Rgb([120, 60, 0]),
        Rgb([84, 90, 0]),
        Rgb([40, 114, 0]),
        Rgb([8, 124, 0]),
        Rgb([0, 118, 40]),
        Rgb([0, 102, 120]),
        Rgb([0, 0, 0]),
        Rgb([0, 0, 0]),
        Rgb([0, 0, 0]),
        Rgb([236, 238, 236]),
        Rgb([76, 154, 236]),
        Rgb([120, 124, 236]),
        Rgb([176, 98, 236]),
        Rgb([228, 84, 236]),
        Rgb([236, 88, 180]),
        Rgb([236, 106, 100]),
        Rgb([212, 136, 32]),
        Rgb([160, 170, 0]),
        Rgb([116, 196, 0]),
        Rgb([76, 208, 32]),
        Rgb([56, 204, 108]),
        Rgb([56, 180, 204]),
        Rgb([60, 60, 60]),
        Rgb([0, 0, 0]),
        Rgb([0, 0, 0]),
        Rgb([236, 238, 236]),
        Rgb([168, 204, 236]),
        Rgb([188, 188, 236]),
        Rgb([212, 178, 236]),
        Rgb([236, 174, 236]),
        Rgb([236, 174, 212]),
        Rgb([236, 180, 176]),
        Rgb([228, 196, 144]),
        Rgb([204, 210, 120]),
        Rgb([180, 222, 120]),
        Rgb([168, 226, 144]),
        Rgb([152, 226, 180]),
        Rgb([160, 214, 228]),
        Rgb([160, 162, 160]),
        Rgb([0, 0, 0]),
        Rgb([0, 0, 0]),
    ]
});

pub static NES_PALLET_SHADER_CONST: Lazy<String> = Lazy::new(|| {
    let color_count = NES_PALLET.len();
    let mut color_list = Vec::new();
    for color in NES_PALLET.iter() {
        color_list.push(format!(
            "vec3<f32>({:.7}, {:.7}, {:.7})",
            linear_f32_from_gamma_u8(color[0]),
            linear_f32_from_gamma_u8(color[1]),
            linear_f32_from_gamma_u8(color[2]),
        ));
    }
    let color_list = color_list.join(",\n");

    format!(
        r#"var<private> NES_PALLET: array<vec3<f32>, {color_count}> = array<vec3<f32>, {color_count}>(
            {color_list}
        );"#,
        color_list = color_list,
        color_count = color_count
    )
});
