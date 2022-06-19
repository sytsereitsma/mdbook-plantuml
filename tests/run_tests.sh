#!/bin/sh

if [ ! -e venv ]; then
  echo Setting up venv
  echo ========================================
  python3 -m venv venv
  venv/bin/pip install -r requirements.txt
  echo
fi

export PATH=$(pwd)/venv/bin/:${PATH}
export PYTHONPATH=$(pwd)/shared_modules

echo Running regression tests
echo ========================================
cd regression_test
../venv/bin/pytest -s -v .
cd -

echo
echo Running e2e tests
echo ========================================
# docker pull plantuml/plantuml-server

DOCKER_ID=`sudo docker ps -qf "ancestor=plantuml/plantuml-server:jetty"`
if [ "${DOCKER_ID}" = "" ]; then
    echo Starting docker plantuml server
    sudo systemctl start docker
    sudo docker run -it -d -p 8080:8080 plantuml/plantuml-server:jetty
else
    echo Docker plantuml server already running
fi

cd e2e_test
../venv/bin/pytest -s -v .
cd -