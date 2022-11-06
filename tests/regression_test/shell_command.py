"""
A little fake application to be used as a plantuml.jar dropin
It is used to capture the arguments plantuml is called with and returns a fake
'image' which stores the command line plantuml would have been called with for
that file.
"""
import sys
import os
import json
import file_locations

if __name__ == "__main__":
    if "-version" in sys.argv:
        print("Shell command 1.2.3")
        sys.exit(0)

    # Generate the dummy image
    img_filename, _ = os.path.splitext(sys.argv[-1])
    if "-tsvg" in sys.argv:
        img_filename += ".svg"
    else:
        img_filename += ".png"

    # Append command output to calls file
    try:
        prev_commands = json.load(open(file_locations.get_shell_calls_file()))
    except IOError:
        prev_commands = []

    if "-pipe" in sys.argv:
        plantuml_code = sys.stdin.read()
        
        # Dump the command line arguments and resulting image location in the
        # 'image file'. The tester can then check these for validity.
        # Do not use print, as this will append \r\n on windows and \n on Linux, which makes testing hard
        sys.stdout.write(" ".join(sys.argv[1:]))

        prev_commands.append({
            "arguments": sys.argv[1:],
            "plantuml-code": plantuml_code
        })
    else:
        # Dump the command line arguments and resulting image location in the
        # 'image file'. The tester can then check these for validity
        # We need to create an image file, so mdbook-plantuml has something to chew
        # on.
        open(img_filename, "w").write(" ".join(sys.argv))

        prev_commands.append({
            "arguments": sys.argv[1:],
            "plantuml-code": open(sys.argv[-1], "rt").read()
        })

    json.dump(
        prev_commands,
        open(file_locations.get_shell_calls_file(), "wt"),
        indent=2
    )

