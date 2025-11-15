{pkgs, ...}: {
  config = {
    devShell = {
      contents = with pkgs; [mdbook];
    };
    programs.zed.enable = true;
    programs.rust.enable = true;
  };
}
