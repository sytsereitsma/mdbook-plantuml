import sys
import os
import json
import file_locations

if __name__ == "__main__":
    # Generate the dummy image
    filename, _ = os.path.splitext(sys.argv[-1])
    if "-tsvg" in sys.argv:
        filename += ".svg"
    else:
        filename += ".png"

    open(filename, "w").write(" ".join(sys.argv))

    open(filename, "w").write(" ".join(sys.argv))

    #Append command output to calls file
    try:
        prev_commands = json.load(open(file_locations.get_shell_calls_file()))
    except IOError:
        prev_commands=[]

    prev_commands.append({
        "arguments": sys.argv[1:],
        "plantuml-code": open(sys.argv[-1], "rt").read().strip()
    })
    json.dump(prev_commands, open(file_locations.get_shell_calls_file(), "wt"), indent=2)

