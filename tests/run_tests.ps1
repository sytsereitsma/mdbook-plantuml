
function IsPlantUMLServerRunning {
    # First we create the request.
    $HTTP_Request = [System.Net.WebRequest]::Create("http://localhost:8080/")

    # We then get a response from the site.
    $HTTP_Response = $HTTP_Request.GetResponse()

    # We then get the HTTP code as an integer.
    $HTTP_Status = [int]$HTTP_Response.StatusCode

    $ok = $false

    If ($HTTP_Status -eq 200) {
        $ok = $true
    }

    # Finally, we clean up the http request by closing it.
    If ($HTTP_Response -ne $null) { $HTTP_Response.Close() }
    
    return $ok
}

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

if (-not (IsPlantUMLServerRunning)) {
    Write-Error "Docker plantuml server not responding, please check the docker container"
    exit 1
}

Push-Location e2e_test
pytest -s -v .
Pop-Location

# Restore environment variables
$env:PATH = $env:PATH
$env:PYTHONPATH = $env:PYTHONPATH
