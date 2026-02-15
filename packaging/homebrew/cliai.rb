class Cliai < Formula
  desc "A powerful CLI assistant powered by local AI (Ollama)"
  homepage "https://github.com/cliai/cliai"
  url "https://github.com/cliai/cliai/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"
  head "https://github.com/cliai/cliai.git", branch: "main"

  depends_on "rust" => :build
  depends_on "openssl@3"

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      CLIAI works best with Ollama for local AI processing.
      Install Ollama: brew install ollama
      
      After installation, run:
        ollama pull mistral
        cliai "hello world"
    EOS
  end

  test do
    assert_match "cliai", shell_output("#{bin}/cliai --version")
  end
end