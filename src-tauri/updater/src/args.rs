use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub old_exe: PathBuf,
    pub new_exe: PathBuf,
    pub pid: u32,
    pub signature: String,
    /// 替换完成后是否重启新版本（默认 true）；静默更新（用户已退出程序）传 --no-relaunch 关闭
    pub relaunch: bool,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        let mut args = std::env::args().skip(1);
        let mut old_exe = None;
        let mut new_exe = None;
        let mut pid = None;
        let mut signature = None;
        let mut relaunch = true;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--old-exe" => {
                    old_exe = Some(PathBuf::from(args.next().ok_or("缺少 --old-exe 值")?))
                }
                "--new-exe" => {
                    new_exe = Some(PathBuf::from(args.next().ok_or("缺少 --new-exe 值")?))
                }
                "--pid" => {
                    pid = Some(
                        args.next()
                            .ok_or("缺少 --pid 值")?
                            .parse()
                            .map_err(|_| "pid 必须是数字")?,
                    )
                }
                "--signature" => signature = Some(args.next().ok_or("缺少 --signature 值")?),
                "--no-relaunch" => relaunch = false,
                _ => return Err(format!("未知参数: {}", arg)),
            }
        }

        Ok(Self {
            old_exe: old_exe.ok_or("缺少 --old-exe")?,
            new_exe: new_exe.ok_or("缺少 --new-exe")?,
            pid: pid.ok_or("缺少 --pid")?,
            signature: signature.ok_or("缺少 --signature")?,
            relaunch,
        })
    }
}
