$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "e:\A-Code\skills-hub\.tauri\agentkit.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
Set-Location "e:\A-Code\skills-hub\frontend"
npm run tauri build
