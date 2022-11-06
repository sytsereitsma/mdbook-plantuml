import os

def get_test_output_dir():
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "test_output")

def get_shell_calls_file():
    return os.path.join(get_test_output_dir(), "shell_calls.txt")

