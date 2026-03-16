class I18nValidate < Formula
  desc "Validate i18n translation files for consistency across 32 formats"
  homepage "https://github.com/i18n-agent/i18n-validate"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/i18n-agent/i18n-validate/releases/download/v#{version}/i18n-validate-aarch64-apple-darwin.tar.gz"
      sha256 "3872787def1d67edd929dd58297f41089599ba676a341a63be55c9931c23dc80"
    else
      url "https://github.com/i18n-agent/i18n-validate/releases/download/v#{version}/i18n-validate-x86_64-apple-darwin.tar.gz"
      sha256 "794b348705c40426b40646ec53a942af26dd3a554be8b97f85ef59c7e9891a00"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/i18n-agent/i18n-validate/releases/download/v#{version}/i18n-validate-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "f16be2d3a6ddc1aa0d4742f98558a4e431115f6d55d0fa5b92b57b1112b316d4"
    else
      url "https://github.com/i18n-agent/i18n-validate/releases/download/v#{version}/i18n-validate-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "cf40f27f0547c972790efa6053f589f29d97d828a7e4b127b1f526ebaffef5fb"
    end
  end

  def install
    bin.install "i18n-validate"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/i18n-validate --version")
  end
end
