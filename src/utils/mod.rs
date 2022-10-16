pub mod dirs;
pub mod init;

#[macro_export]
macro_rules! log_err {
  ($result: expr) => {
    if let Err(err) = $result {
      log::error!(target: "app", "{err}");
    }
  };
}
