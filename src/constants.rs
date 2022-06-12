use egui::{color::linear_f32_from_gamma_u8, Color32};
use once_cell::sync::Lazy;

// Uncomment if we need clipboard support later
// pub(crate) static CLIPBOARD: Lazy<Mutex<arboard::Clipboard>> =
//     Lazy::new(|| Mutex::new(arboard::Clipboard::new().expect("Access clipboard")));

/// NES color pallet
pub static NES_PALLET: Lazy<[Color32; 64]> = Lazy::new(|| {
    [
        Color32::from_rgb(84, 84, 84),
        Color32::from_rgb(0, 30, 116),
        Color32::from_rgb(8, 16, 144),
        Color32::from_rgb(48, 0, 136),
        Color32::from_rgb(68, 0, 100),
        Color32::from_rgb(92, 0, 48),
        Color32::from_rgb(84, 4, 0),
        Color32::from_rgb(60, 24, 0),
        Color32::from_rgb(32, 42, 0),
        Color32::from_rgb(8, 58, 0),
        Color32::from_rgb(0, 64, 0),
        Color32::from_rgb(0, 60, 0),
        Color32::from_rgb(0, 50, 60),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(152, 150, 152),
        Color32::from_rgb(8, 76, 196),
        Color32::from_rgb(48, 50, 236),
        Color32::from_rgb(92, 30, 228),
        Color32::from_rgb(136, 20, 176),
        Color32::from_rgb(160, 20, 100),
        Color32::from_rgb(152, 34, 32),
        Color32::from_rgb(120, 60, 0),
        Color32::from_rgb(84, 90, 0),
        Color32::from_rgb(40, 114, 0),
        Color32::from_rgb(8, 124, 0),
        Color32::from_rgb(0, 118, 40),
        Color32::from_rgb(0, 102, 120),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(236, 238, 236),
        Color32::from_rgb(76, 154, 236),
        Color32::from_rgb(120, 124, 236),
        Color32::from_rgb(176, 98, 236),
        Color32::from_rgb(228, 84, 236),
        Color32::from_rgb(236, 88, 180),
        Color32::from_rgb(236, 106, 100),
        Color32::from_rgb(212, 136, 32),
        Color32::from_rgb(160, 170, 0),
        Color32::from_rgb(116, 196, 0),
        Color32::from_rgb(76, 208, 32),
        Color32::from_rgb(56, 204, 108),
        Color32::from_rgb(56, 180, 204),
        Color32::from_rgb(60, 60, 60),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(236, 238, 236),
        Color32::from_rgb(168, 204, 236),
        Color32::from_rgb(188, 188, 236),
        Color32::from_rgb(212, 178, 236),
        Color32::from_rgb(236, 174, 236),
        Color32::from_rgb(236, 174, 212),
        Color32::from_rgb(236, 180, 176),
        Color32::from_rgb(228, 196, 144),
        Color32::from_rgb(204, 210, 120),
        Color32::from_rgb(180, 222, 120),
        Color32::from_rgb(168, 226, 144),
        Color32::from_rgb(152, 226, 180),
        Color32::from_rgb(160, 214, 228),
        Color32::from_rgb(160, 162, 160),
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(0, 0, 0),
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
