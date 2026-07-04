class Sharecli < Formula
  desc "Process hypervisor and CLI toolkit"
  homepage "https://github.com/KooshaPari/sharecli"
  version "0.2.0"
  url "https://github.com/KooshaPari/sharecli/releases/download/v0.2.0/sharecli-aarch64-apple-darwin.tar.gz"
  sha256 "PLACEHOLDER"
  def install
    bin.install "sharecli"
  end
  test do
    system "#{bin}/sharecli", "--version"
  end
end