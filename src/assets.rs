use include_dir::{Dir, include_dir};

pub static FONTS: &Dir = &include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");

pub static EYEBALLS: &Dir = &include_dir!("$CARGO_MANIFEST_DIR/assets/eyeballs");
pub static WINGS: &Dir = &include_dir!("$CARGO_MANIFEST_DIR/assets/wings");
