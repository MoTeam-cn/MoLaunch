//! frpc 进程管理：启动/停止/状态查询 + 日志捕获 + 退出监听
//! 子模块：start（启动）/ stop（停止）/ status（状态查询）/
//! capture（stdout/stderr 捕获）/ log（日志文件读取）/ state（运行状态）

mod capture;
mod log;
mod start;
mod state;
mod status;
mod stop;

pub use log::{list_log_files, read_log_file};
pub use start::start_tunnel;
use state::{FrpcHandle, RUNNING};
pub use status::{get_tunnel_status, list_tunnels_with_status};
pub use stop::stop_tunnel;