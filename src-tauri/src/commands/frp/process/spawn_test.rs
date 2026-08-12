use crate::commands::frp::process::spawn::build_command_args;

#[test]
fn token_inline_concatenated() {
    let args = build_command_args("{frpc} -t 17062:{token}", "jmbscabc");
    assert_eq!(args, vec!["-t", "17062:jmbscabc"]);
}

#[test]
fn token_as_standalone_word() {
    let args = build_command_args("{frpc} -u {token} -p 123", "abc");
    assert_eq!(args, vec!["-u", "abc", "-p", "123"]);
}

#[test]
fn token_in_middle_with_trailing_args() {
    let args = build_command_args("{frpc} -t 17062:{token} --server relay.example.com", "abc");
    assert_eq!(
        args,
        vec!["-t", "17062:abc", "--server", "relay.example.com"]
    );
}

#[test]
fn empty_token_skips_standalone_placeholder() {
    let args = build_command_args("{frpc} -t 17062:{token}", "");
    assert_eq!(args, vec!["-t", "17062:"]);
    assert_eq!(build_command_args("{frpc} -t {token}", ""), vec!["-t"]);
}
