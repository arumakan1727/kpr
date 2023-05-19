use kpr_webclient::Platform;

pub fn authtoken_filename(platform: Platform) -> String {
    format!("{}-auth.json", platform.lowercase())
}
