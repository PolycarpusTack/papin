# Start the Papin application
Write-Host "Starting Papin application..." -ForegroundColor Green

# Navigate to project directory
Set-Location C:\Projects\Papin

# Attempt to start the frontend
Write-Host "Starting frontend..." -ForegroundColor Cyan
Set-Location C:\Projects\Papin\src-frontend
npm run dev

# Wait for frontend to start
Start-Sleep -Seconds 5

# In a new PowerShell window, start the backend
Write-Host "Starting backend in a new window..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-Command cd C:\Projects\Papin\src-tauri; cargo run"

Write-Host "Application started successfully!" -ForegroundColor Green
