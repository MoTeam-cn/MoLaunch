use super::*;

#[test]
fn extract_class_name_from_log() {
    let log =
        "[ERROR] java.lang.ClassNotFoundException: net.fabricmc.loader.impl.launch.knot.KnotClient";
    let name = extract_class_name(log, "classnotfoundexception");
    assert!(name.is_some());
    assert!(name.unwrap().contains("net.fabricmc"));
}
