import json
import sys
import os

if __name__ == "__main__":
    if "supports" in sys.argv:
        sys.exit(0)

    json_data = sys.stdin.read()
    book = json.loads(json_data)

    # Correct the book's root
    book[0]["root"] = "."
    book[0]["config"]["book"]["src"] = "."

    # Replace this preprocessor with the plantuml preprocessor
    book[0]["config"]["preprocessor"] = {
        "plantuml": {}
    }
    open("../book.json", "w").write(json.dumps(book, indent="  "))

    sys.stderr.write("Successfully generated ../book.json, but BOOK BUILDING WILL BE FORCED TO FAIL.\n")
    sys.exit(-1)

