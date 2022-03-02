[ -d venv ] && rmdir /s /q venv
py -3 -m virtualenv venv
venv\Scripts\pip install -r requirements.txt

set __orgpath=%PATH%
set PATH=..\venv\Scripts\;%PATH%
set PYTHONPATH=%cd%\shared_modules
cd regression_test
nosetests -s -v .
cd ..
set PATH=%__orgpath%
