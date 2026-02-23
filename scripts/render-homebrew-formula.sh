#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 9 ]]; then
  echo "usage: $0 <version> <repo_owner> <repo_name> <mac_arm_url> <mac_arm_sha> <mac_intel_url> <mac_intel_sha> <linux_url> <linux_sha>" >&2
  exit 1
fi

version="$1"
repo_owner="$2"
repo_name="$3"
mac_arm_url="$4"
mac_arm_sha="$5"
mac_intel_url="$6"
mac_intel_sha="$7"
linux_url="$8"
linux_sha="$9"

cat <<EOF
class Engram < Formula
  desc "AI agent memory system with CLI and MCP server"
  homepage "https://github.com/${repo_owner}/${repo_name}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "${mac_arm_url}"
      sha256 "${mac_arm_sha}"
    else
      url "${mac_intel_url}"
      sha256 "${mac_intel_sha}"
    end
  end

  on_linux do
    url "${linux_url}"
    sha256 "${linux_sha}"
  end

  def install
    bin.install "engram"
    pkgshare.install "README.md", "LICENSE"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/engram --version")
  end
end
EOF
