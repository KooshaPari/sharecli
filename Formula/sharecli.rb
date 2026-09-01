# Homebrew formula for sharecli.
#
# Bottle digest sourced from GitHub Release v0.3.0 asset
# `sharecli-aarch64-apple-darwin.tar.gz` (attached 2026-07-13).
# Recompute on each release:
#   gh release download vX.Y.Z -p 'sharecli-aarch64-apple-darwin.tar.gz'
#   shasum -a 256 sharecli-aarch64-apple-darwin.tar.gz
#
# HEAD install (no bottle):
#   brew install --HEAD Formula/sharecli.rb
class Sharecli < Formula
  desc "Shared CLI process manager for multi-project agent orchestration"
  homepage "https://github.com/KooshaPari/sharecli"
  # Keep in sync with Cargo.toml package.version / latest gh release tag.
  version "0.3.0"
  url "https://github.com/KooshaPari/sharecli/releases/download/v0.3.0/sharecli-aarch64-apple-darwin.tar.gz"
  sha256 "bc97738004d737eeb5b3ae978b42bd85f2976ff4d0987de8b4f9e6dfd0806a61"

  head do
    url "https://github.com/KooshaPari/sharecli.git", branch: "main"
    depends_on "rust" => :build
  end

  def install
    if build.head?
      system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    else
      bin.install "sharecli"
    end
  end

  test do
    system "#{bin}/sharecli", "--version"
  end
end
