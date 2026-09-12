$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "e:\A-Code\AgentKit\.tauri\agentkit.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
Set-Location "e:\A-Code\AgentKit\frontend"
npm run tauri build
