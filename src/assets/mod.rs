use iced::widget::image;

/// FerrumFinance Logo 1 - Main branding logo
pub const LOGO_1: &[u8] = include_bytes!("ferrum-finance-logo-1.png");

/// FerrumFinance Logo 2 - Alternative branding logo
pub const LOGO_2: &[u8] = include_bytes!("ferrum-finance-logo-2.png");

/// FerrumFinance Logo 3 - Compact logo variant
pub const LOGO_3: &[u8] = include_bytes!("ferrum-finance-logo-3.png");

/// FerrumFinance Logo 4 - Icon-style logo
pub const LOGO_4: &[u8] = include_bytes!("ferrum-finance-logo-4.png");

/// Get a logo handle by index (1-4)
pub fn get_logo_handle(logo_index: usize) -> image::Handle {
    match logo_index {
        1 => image::Handle::from_memory(LOGO_1),
        2 => image::Handle::from_memory(LOGO_2),
        3 => image::Handle::from_memory(LOGO_3),
        4 => image::Handle::from_memory(LOGO_4),
        _ => image::Handle::from_memory(LOGO_1), // Default to logo 1
    }
}

/// Get the main splash screen logo (Logo 1)
pub fn get_splash_logo() -> image::Handle {
    get_logo_handle(1) // Use logo 1 for splash screen
}

/// Get the application header logo (Logo 2)
pub fn get_app_logo() -> image::Handle {
    get_logo_handle(2) // Use logo 2 for main app header
}