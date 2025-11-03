cask "maestro" do
  version "0.4.4"
  sha256 "720f48c4f19cf2e99db2d5a72c95dd6fe6334fb46866d6b3889295ebb77e7fd3"

  url "https://github.com/trly/maestro/releases/download/v#{version}/Maestro_#{version}_aarch64.dmg"
  name "Maestro"
  desc "AI Prompt Orchestrator for multi-repo workflows"
  homepage "https://github.com/trly/maestro"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Maestro.app"

  zap trash: [
    "~/Library/Application Support/dev.trly.maestro",
    "~/Library/Logs/dev.trly.maestro",
  ]
end
