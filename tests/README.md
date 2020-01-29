# The testers

The tests are written as python unit tetss and call on teh executables.

## Regression tester
The regression tester calls the mdbook-plantuml executable (all features
enabled) directly (bypassing mdbook).
This makes it sensitive to changes in mdbook, but gives a greater level of control
over the tests. It uses a fake plantuml script to inject errors and capture the
preprocessor output.

The ```./regression_test/regenerate_book_json/regeneration_preprocessor.py```
scripts creates a new template book in ```./regression_test/book.json```. It simply
calls mdbook with a fake preprocessor to catch the json book coming from mdbook itself.

You should run the regeneration_preprocessor whenever the mdbook version changes.

## End to end (E2E) tests
These test the full processing pipeline with all feature combinations and show
the pages in your browser.

This is a bit of a pain, but the image names are randomly generated, so using a
diff tool is not (easily) possible..

## Running the tests
First create the virtual environment (assumes you have the python launcher):

From the tests directory in your console:

### Windows
```
py -3 -m virtualenv venv
venv\Scripts\activate
pip install -r requirements.txt
set PYTHONPATH=%cd%\shared_modules
```

After that run the tests:
```sh
cd regression_tester
nosetests .
```

And for the end to end tests:
```sh
cd e2e_tests
nosetests .
```
