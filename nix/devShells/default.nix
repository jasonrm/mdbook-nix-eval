{pkgs, ...}: {
  config = {
    devShell = {
      contents = with pkgs; [mdbook clippy];
    };
    programs.zed.enable = true;
    programs.rust.enable = true;
  };
}
