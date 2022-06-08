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
        // 00
        Rgb([128, 128, 128]),
        Rgb([0, 61, 166]),
        Rgb([0, 18, 176]),
        Rgb([68, 0, 150]),
        Rgb([161, 0, 94]),
        Rgb([199, 0, 40]),
        Rgb([186, 6, 0]),
        Rgb([140, 23, 0]),
        Rgb([92, 47, 0]),
        Rgb([16, 69, 0]),
        Rgb([5, 74, 0]),
        Rgb([0, 71, 46]),
        Rgb([0, 65, 102]),
        Rgb([3, 3, 3]),
        Rgb([3, 3, 3]),
        Rgb([3, 3, 3]),
        // 10
        Rgb([199, 199, 199]),
        Rgb([0, 119, 255]),
        Rgb([33, 85, 255]),
        Rgb([130, 55, 250]),
        Rgb([235, 47, 181]),
        Rgb([255, 41, 80]),
        Rgb([255, 34, 0]),
        Rgb([214, 50, 0]),
        Rgb([196, 98, 0]),
        Rgb([53, 128, 0]),
        Rgb([5, 143, 0]),
        Rgb([0, 138, 85]),
        Rgb([0, 153, 204]),
        Rgb([33, 33, 33]),
        Rgb([3, 3, 3]),
        Rgb([3, 3, 3]),
        // 20
        Rgb([255, 255, 255]),
        Rgb([15, 215, 255]),
        Rgb([105, 162, 255]),
        Rgb([212, 128, 255]),
        Rgb([255, 69, 243]),
        Rgb([255, 97, 139]),
        Rgb([255, 136, 51]),
        Rgb([255, 156, 18]),
        Rgb([250, 188, 32]),
        Rgb([159, 227, 14]),
        Rgb([43, 240, 53]),
        Rgb([12, 240, 164]),
        Rgb([5, 251, 255]),
        Rgb([94, 94, 94]),
        Rgb([13, 13, 13]),
        Rgb([13, 13, 13]),
        // 30
        Rgb([255, 255, 255]),
        Rgb([166, 252, 255]),
        Rgb([179, 236, 255]),
        Rgb([218, 171, 235]),
        Rgb([255, 168, 249]),
        Rgb([255, 171, 179]),
        Rgb([255, 210, 176]),
        Rgb([255, 239, 166]),
        Rgb([255, 247, 156]),
        Rgb([215, 232, 149]),
        Rgb([166, 237, 157]),
        Rgb([162, 242, 218]),
        Rgb([153, 255, 252]),
        Rgb([221, 221, 221]),
        Rgb([17, 17, 17]),
        Rgb([17, 17, 17]),
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
