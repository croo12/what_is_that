fn main() {
  #[cfg(target_os = "windows")]
  {
      // Packet.lib 파일이 있는 경로
      let sdk_path = r"C:\Users\jimmy\my_cli_tool\lib";
      println!("cargo:rustc-link-search=native={}", sdk_path);
      println!("cargo:rustc-link-lib=static=Packet");
  }
}
