class DnKernel < Formula
  desc "Terminal-first repository review and scanning tool"
  homepage "https://github.com/ali-baneshi/dn-kernel"
  version "0.1.0"
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/ali-baneshi/dn-kernel/releases/download/v0.1.0/dn-kernel-v0.1.0-aarch64-apple-darwin.tar.gz"
    sha256 "REPLACE_ARM_MAC_SHA256"
  elsif OS.mac?
    url "https://github.com/ali-baneshi/dn-kernel/releases/download/v0.1.0/dn-kernel-v0.1.0-x86_64-apple-darwin.tar.gz"
    sha256 "REPLACE_X64_MAC_SHA256"
  else
    url "https://github.com/ali-baneshi/dn-kernel/releases/download/v0.1.0/dn-kernel-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_LINUX_SHA256"
  end

  def install
    bin.install "dn-cli" => "dn-kernel"
  end

  test do
    system "#{bin}/dn-kernel", "--version"
  end
end
