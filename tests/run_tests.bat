@echo off
if not exist venv (
  echo Setting up venv
  echo ========================================
  py -3 -m virtualenv venv
  venv\Scripts\pip install -r requirements.txt
  echo.
)

set __ORGPATH=%PATH%
set PATH=%cd%\venv\Scripts\;%PATH%
set __ORGPYTHONPATH=%PYTHONPATH%
set PYTHONPATH=%cd%\shared_modules

echo Running regression tests
echo ========================================
cd regression_test
nosetests -s -v .
cd ..

echo.
echo Running e2e tests
echo ========================================
cd e2e_test
rem docker pull plantuml/plantuml-server
set __DOCKER_RUNNING=
for /f %%i in ('docker ps -qf "ancestor=plantuml/plantuml-server:jetty"') do set __DOCKER_RUNNING=1
if [%__DOCKER_RUNNING%] EQU [] (
    echo Starting docker plantuml server
    docker run -it -d -p 8080:8080 plantuml/plantuml-server:jetty
) else (
    echo Docker plantuml server already running
)

nosetests -s -v .
cd ..

set PATH=%__ORGPATH%
set PYTHONPATH=%__ORGPYTHONPATH%
set __DOCKER_RUNNING=
set __ORGPATH=
set __ORGPYTHONPATH=
