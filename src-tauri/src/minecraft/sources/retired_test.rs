//! 退役地址还原测试

use super::normalize;

#[test]
fn test_备用CDN还原到官方() {
    assert_eq!(
        normalize("https://cdn-alt.modrinth.com/data/abc/versions/1/file.jar"),
        "https://cdn.modrinth.com/data/abc/versions/1/file.jar"
    );
}

#[test]
fn test_旧第三方镜像按路径前缀分流() {
    assert_eq!(
        normalize("https://mod.tianpao.top/data/abc/versions/1/file.jar"),
        "https://cdn.modrinth.com/data/abc/versions/1/file.jar"
    );
    assert_eq!(
        normalize("https://mod.tianpao.top/files/123/456/a.jar"),
        "https://edge.forgecdn.net/files/123/456/a.jar"
    );
    assert_eq!(
        normalize("https://mod.tianpao.top/media/attachments/123/icon.png"),
        "https://media.forgecdn.net/attachments/123/icon.png"
    );
}

#[test]
fn test_查询串原样保留() {
    assert_eq!(
        normalize("https://mod.tianpao.top/files/1/2/a.jar?x=1&y=%2B#frag"),
        "https://edge.forgecdn.net/files/1/2/a.jar?x=1&y=%2B#frag"
    );
}

#[test]
fn test_不认识的主机或路径原样返回() {
    assert_eq!(
        normalize("https://cdn.modrinth.com/data/a/b.jar"),
        "https://cdn.modrinth.com/data/a/b.jar"
    );
    // 主机对但路径对不上：不猜
    assert_eq!(
        normalize("https://mod.tianpao.top/other/x.jar"),
        "https://mod.tianpao.top/other/x.jar"
    );
    // 非 https 不处理
    assert_eq!(normalize("http://x/y"), "http://x/y");
}
