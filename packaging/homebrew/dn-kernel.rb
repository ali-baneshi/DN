class DnKernel < Formula
  desc "Terminal-first repository review and scanning tool"
  homepage "https://github.com/ali-baneshi/dn-kernel"
  version "1.1.0"
  if Hardware::CPU.arm?
    url "https://github.com/ali-baneshi/dn-kernel/releases/download/v1.1.0/dn-kernel-v1.1.0-aarch64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_AARCH64_LINUX_SHA256"
  else
    url "https://github.com/ali-baneshi/dn-kernel/releases/download/v1.1.0/dn-kernel-v1.1.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_X86_64_LINUX_SHA256"
  end

  def install
    bin.install "dn-cli" => "dn-kernel"
    bin.install "dn-worker-c"
  end

  test do
    system "#{bin}/dn-kernel", "--version"
    system "#{bin}/dn-worker-c", <<~EOS
      {"protocol_version":"1.0.0","request_id":"hello","method":"hello","params":{}}
    EOS
  end
end
