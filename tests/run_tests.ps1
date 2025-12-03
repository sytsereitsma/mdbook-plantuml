# PowerShell version of run_tests.bat
# Setup virtual environment if not exists
if (!(Test-Path "venv")) {
    Write-Host "Setting up venv"
    Write-Host "========================================"
    py -3 -m virtualenv venv
    .\venv\Scripts\pip install -r requirements.txt
    Write-Host ""
}

$WD = Get-Location
$env:PATH = "$PWD\venv\Scripts;" + $env:PATH
$env:PYTHONPATH = "$PWD\shared_modules"

Write-Host "Running regression tests"
Write-Host "========================================"
Push-Location regression_test
pytest -s -v .
Pop-Location

Write-Host ""
Write-Host "Running e2e tests"
Write-Host "========================================"

# Check if Docker daemon is running
try {
    docker info | Out-Null
} catch {
    Write-Error "Docker daemon not running, please start the docker daemon"
    exit 1
}

# Check if plantuml/plantuml-server:jetty is running
$dockerRunning = docker ps -qf "ancestor=plantuml/plantuml-server:jetty"
if (-not $dockerRunning) {
    Write-Host "Starting docker plantuml server"
    docker run -it -d -p 8080:8080 plantuml/plantuml-server:jetty
} else {
    Write-Host "Docker plantuml server already running"
}

Push-Location e2e_test
pytest -s -v .
Pop-Location

# Restore environment variables
$env:PATH = $env:PATH
$env:PYTHONPATH = $env:PYTHONPATH
