use egui::mutex::Mutex;
use once_cell::sync::Lazy;
use palette::Srgb;

pub(crate) static CLIPBOARD: Lazy<Mutex<arboard::Clipboard>> =
    Lazy::new(|| Mutex::new(arboard::Clipboard::new().expect("Access clipboard")));

// TODO: Use egui::Color32 instead of Srgb<u8>
/// NES color palette
pub(crate) static NES_PALETTE_RGB: Lazy<[Srgb<u8>; 64]> = Lazy::new(|| {
    [
        // 00
        Srgb::new(128, 128, 128),
        Srgb::new(0, 61, 166),
        Srgb::new(0, 18, 176),
        Srgb::new(68, 0, 150),
        Srgb::new(161, 0, 94),
        Srgb::new(199, 0, 40),
        Srgb::new(186, 6, 0),
        Srgb::new(140, 23, 0),
        Srgb::new(92, 47, 0),
        Srgb::new(16, 69, 0),
        Srgb::new(5, 74, 0),
        Srgb::new(0, 71, 46),
        Srgb::new(0, 65, 102),
        Srgb::new(3, 3, 3),
        Srgb::new(3, 3, 3),
        Srgb::new(3, 3, 3),
        // 10
        Srgb::new(199, 199, 199),
        Srgb::new(0, 119, 255),
        Srgb::new(33, 85, 255),
        Srgb::new(130, 55, 250),
        Srgb::new(235, 47, 181),
        Srgb::new(255, 41, 80),
        Srgb::new(255, 34, 0),
        Srgb::new(214, 50, 0),
        Srgb::new(196, 98, 0),
        Srgb::new(53, 128, 0),
        Srgb::new(5, 143, 0),
        Srgb::new(0, 138, 85),
        Srgb::new(0, 153, 204),
        Srgb::new(33, 33, 33),
        Srgb::new(3, 3, 3),
        Srgb::new(3, 3, 3),
        // 20
        Srgb::new(255, 255, 255),
        Srgb::new(15, 215, 255),
        Srgb::new(105, 162, 255),
        Srgb::new(212, 128, 255),
        Srgb::new(255, 69, 243),
        Srgb::new(255, 97, 139),
        Srgb::new(255, 136, 51),
        Srgb::new(255, 156, 18),
        Srgb::new(250, 188, 32),
        Srgb::new(159, 227, 14),
        Srgb::new(43, 240, 53),
        Srgb::new(12, 240, 164),
        Srgb::new(5, 251, 255),
        Srgb::new(94, 94, 94),
        Srgb::new(13, 13, 13),
        Srgb::new(13, 13, 13),
        // 30
        Srgb::new(255, 255, 255),
        Srgb::new(166, 252, 255),
        Srgb::new(179, 236, 255),
        Srgb::new(218, 171, 235),
        Srgb::new(255, 168, 249),
        Srgb::new(255, 171, 179),
        Srgb::new(255, 210, 176),
        Srgb::new(255, 239, 166),
        Srgb::new(255, 247, 156),
        Srgb::new(215, 232, 149),
        Srgb::new(166, 237, 157),
        Srgb::new(162, 242, 218),
        Srgb::new(153, 255, 252),
        Srgb::new(221, 221, 221),
        Srgb::new(17, 17, 17),
        Srgb::new(17, 17, 17),
    ]
});
