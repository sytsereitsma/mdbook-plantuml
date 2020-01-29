import subprocess
import os

def _build(extra_args):
    args = [
        "cargo",
        "build",
        "--release"
    ]

    args += extra_args

    module_dir = os.path.dirname(os.path.abspath(__file__))
    build_dir = os.path.abspath(os.path.join(module_dir, ".."))

    proc = subprocess.Popen(args,
        cwd=build_dir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE)

    stdout, stderr = proc.communicate()
    if proc.returncode != 0:
        print("Error building mdbook-plantuml (exit code {}) with args: {}"
            .format(proc.returncode, " ".join(args)))
        print("Stdout:\n===============================")
        print(stdout)

        print("Stderr:\n===============================")
        print(stderr)

    return proc.returncode == 0

def build_full():
    return _build([])

def build_shell():
    return _build(["--no-default-features"])

def build_http_server():
    return _build(["--no-default-features", "--features", "plantuml-server"])

def build_https_server():
    return _build(["--no-default-features", "--features", "plantuml-ssl-server"])

