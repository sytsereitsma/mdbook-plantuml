[ -d venv ] && rmdir /s /q venv
py -3 -m virtualenv venv
venv\Scripts\pip install -r requirements.txt
set PYTHONPATH=%cd%\shared_modules
cd regression_test
..\venv\Scripts\nosetests -s -v .
cd ..
