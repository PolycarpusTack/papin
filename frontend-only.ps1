# Start only the frontend of the Papin application
Write-Host "Starting Papin frontend only..." -ForegroundColor Green

# Navigate to frontend directory
Set-Location C:\Projects\Papin\src-frontend

# Start the frontend
npm run dev

Write-Host "Frontend started successfully!" -ForegroundColor Green
