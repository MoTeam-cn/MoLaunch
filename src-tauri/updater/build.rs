use winres::WindowsResource;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        res.set("FileDescription", "MoLaunch Updater");
        res.set("ProductName", "MoLaunch");
        res.set("LegalCopyright", "Copyright (c) 2026 MoTeam");
        res.set_icon("../Images/icon.ico");
        res.compile().unwrap();
    }
}
