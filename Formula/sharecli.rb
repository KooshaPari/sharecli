# Homebrew formula for sharecli.
#
# PLACEHOLDER removal plan (sha256 unknown — GitHub Releases v0.3.0 has no
# attached assets as of 2026-07-10):
#   1. Tag/publish a release that uploads `sharecli-aarch64-apple-darwin.tar.gz`
#      (see `.github/workflows/release.yml` linux+mac artifact matrix).
#   2. Fetch the asset digest:
#        gh release download v0.3.0 -p 'sharecli-aarch64-apple-darwin.tar.gz'
#        shasum -a 256 sharecli-aarch64-apple-darwin.tar.gz
#      or: gh api repos/KooshaPari/sharecli/releases/tags/v0.3.0 \
#            --jq '.assets[] | select(.name|test("aarch64-apple-darwin")) | .digest'
#   3. Replace sha256 "PLACEHOLDER" below with the real hex digest.
#   4. Bump `version` / `url` in lockstep with Cargo.toml on each release.
#
# Until release assets exist, install from git HEAD:
#   brew install --HEAD Formula/sharecli.rb
class Sharecli < Formula
  desc "Shared CLI process manager for multi-project agent orchestration"
  homepage "https://github.com/KooshaPari/sharecli"
  # Keep in sync with Cargo.toml package.version / latest gh release tag.
  version "0.3.0"
  url "https://github.com/KooshaPari/sharecli/releases/download/v0.3.0/sharecli-aarch64-apple-darwin.tar.gz"
  sha256 "PLACEHOLDER"

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
