import json
import sys
import os

if __name__ == "__main__":
    if "supports" in sys.argv:
        sys.exit(0)

    json_data = sys.stdin.read()
    book = json.loads(json_data)

    # Correct the book's root
    tester_root = os.path.dirname(__file__)
    tester_root = os.path.abspath(os.path.join(tester_root, ".."))
    book[0]["root"] = tester_root

    # output stuff to test_output rather than 'src'
    book[0]["config"]["book"]["src"] = "test_output"

    # Replace this preprocessor with the plantuml preprocessor
    book[0]["config"]["preprocessor"] = {
        "plantuml": {}
    }
    open("../book.json", "w").write(json.dumps(book, indent="  "))

    sys.stderr.write("Successfully generated ../book.json, but BOOK BUILDING WILL BE FORCED TO FAIL.\n")
    sys.exit(-1)

