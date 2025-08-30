# The testers

The tests are written as python unit tests and call on the executables.

## Regression tester
The regression tester calls the mdbook-plantuml executable (all features
enabled) directly (bypassing mdbook).
This makes it sensitive to changes in mdbook, but gives a greater level of control
over the tests. It uses a fake plantuml script (fake_plantuml.py) to inject errors
and capture the preprocessor output.

The ```./regression_test/regenerate_book_json/regeneration_preprocessor.py```
scripts creates a new template book in ```./regression_test/book.json```. It simply
calls mdbook with a fake preprocessor to catch the json book coming from mdbook itself.

You should run the regeneration_preprocessor whenever the mdbook version changes:
```
cd regenerate_book_json
mdbook build
```

## End to end (E2E) tests
These test the full processing pipeline with all feature combinations (the mdbook-plantuml
executable is rebuilt with different features enabled/disabled) and show the pages in your browser.

This is a bit of a pain, but the image names are randomly generated, so using a
diff tool is not (easily) possible..

## Prerequisites for running the tests

* Docker client
* mdbook
* plantuml

> Make sure you have a docker daemon running.
 
 ```
 docker pull plantuml/plantuml-server:jetty
 docker run -d -p 8080:8080 plantuml/plantuml-server:jetty
 ```

### Windows
Simply run ```run_tests.bat```

Or, manually:
```
py -3 -m venv venv
venv\Scripts\activate
pip install -r requirements.txt
set PYTHONPATH=%cd%\shared_modules
```

After that run the tests:
```sh
cd regression_tester
pytest
```

And for the end to end tests:
```sh
cd e2e_tests
pytest
```

### Linux
Simply run ```run_tests.sh```

Or, manually:
```
python3 -m venv venv
. venv\bin\activate
pip install -r requirements.txt
export PYTHONPATH=$(pwd)\shared_modules
```

After that run the tests:
```sh
cd regression_tester
pytest
```

And for the end to end tests:
```sh
cd e2e_tests
pytest
```
