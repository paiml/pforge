class Pforge < Formula
  desc "Zero-boilerplate MCP server framework with declarative YAML configuration"
  homepage "https://github.com/paiml/pforge"
  url "https://github.com/paiml/pforge/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/paiml/pforge.git", branch: "main"

  depends_on "rust" => :build
  depends_on "pkg-config" => :build
  depends_on "openssl@3"

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/pforge-cli")

    # Install shell completions
    generate_completions_from_executable(bin/"pforge", "completions")

    # Install examples
    (share/"pforge/examples").install Dir["examples/*"]

    # Install documentation
    doc.install "README.md", "USER_GUIDE.md", "ARCHITECTURE.md"
  end

  test do
    # Test version
    assert_match "pforge", shell_output("#{bin}/pforge --version")

    # Test help
    assert_match "USAGE", shell_output("#{bin}/pforge --help")

    # Test scaffold
    system bin/"pforge", "new", "test-server", "--template", "minimal"
    assert_predicate testpath/"test-server/pforge.yaml", :exist?
  end
end
