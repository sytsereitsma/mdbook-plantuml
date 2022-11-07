import unittest
import subprocess
import os
import shutil
import preprocessor_builder
import webbrowser
import platform
import time
import toml


def get_module_dir():
    return os.path.dirname(os.path.abspath(__file__))


def change_output_dir(book_toml, output_dir):
    data = toml.load(open(book_toml))
    data["build"]["build-dir"] = output_dir
    toml.dump(data, open(book_toml, "w"))


def _build_book(book_name, open_browser=True, output_dir=None, clean=True):
    module_dir = get_module_dir()
    src_toml = os.path.join(module_dir, book_name)
    target_toml = os.path.join(module_dir, "book.toml")
    shutil.copy(src_toml, target_toml)

    if output_dir is None:
        book_output_dir = os.path.join(module_dir, os.path.splitext(book_name)[0])
    else:
        book_output_dir = os.path.join(module_dir, output_dir)
        change_output_dir(target_toml, output_dir)

    if clean and os.path.isdir(book_output_dir):
        shutil.rmtree(book_output_dir)

    if clean and os.path.isdir(os.path.join("src", "mdbook-plantuml-img")):
        shutil.rmtree(os.path.join("src", "mdbook-plantuml-img"))

    if clean and os.path.isdir(".mdbook-plantuml-cache"):
        shutil.rmtree(".mdbook-plantuml-cache")

    preprocessor_dir = os.path.join(module_dir, "..", "..", "target", "release")
    env = os.environ
    if platform.system() == "Windows":
        env["PATH"] = preprocessor_dir + ";" + env["PATH"]
    else:
        env["PATH"] = preprocessor_dir + ":" + env["PATH"]

    args = ["mdbook", "build"]
    proc = subprocess.Popen(args,
        cwd=module_dir,
        env=env,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE)

    stdout, stderr = proc.communicate()
    if proc.returncode != 0:
        print("Error building book {} (exit code {}) with args: {}"
            .format(book_name, proc.returncode, " ".join(args)))
        print("Stdout:\n===============================")
        print(stdout)

        print("Stderr:\n===============================")
        print(stderr)
    elif open_browser:
        index_html = os.path.join(book_output_dir, "index.html")
        webbrowser.open('file://' + index_html)

    return proc.returncode == 0


class TestEndToEndServer(unittest.TestCase):
    def test_http_server(self):
        assert preprocessor_builder.build_http_server()
        assert _build_book("plantuml_server.toml")
        assert _build_book("plantuml_server_datauri.toml")

    def test_https_server(self):
        assert preprocessor_builder.build_https_server()
        #TODO: Run https server somewhere
        assert _build_book("plantuml_server.toml", output_dir="plantuml_ssl_server")


class TestEndToEndShell(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert preprocessor_builder.build_shell()

    def test_shell(self):
        assert _build_book("plantuml_shell.toml")
        assert _build_book("plantuml_shell_not_piped.toml")

    def test_clickable(self):
        assert _build_book("plantuml_shell_clickable.toml")

    def test_data_uris(self):
        assert _build_book("plantuml_shell_datauri.toml")

    def test_dir_cleaner(self):
        assert _build_book("plantuml_shell.toml", output_dir="plantuml_dir_cleaner")
        assert _build_book("plantuml_shell.toml", output_dir="plantuml_dir_cleaner_no_clean", clean=False)

    def test_cache(self):
        cache_dir = os.path.join(get_module_dir(), ".plantuml-cache")
        if os.path.isdir(cache_dir):
            shutil.rmtree(cache_dir)

        uncached_start = time.time()
        assert _build_book("plantuml_cache.toml", False)
        uncached_time = time.time() - uncached_start

        cached_start = time.time()
        assert _build_book("plantuml_cache.toml", clean=False)
        cached_time = time.time() - cached_start

        # TODO: This is lame. Use a custom plantuml command the second time to
        # make sure it is not called
        self.assertLess(cached_time, uncached_time / 4)

    def test_shell_has_no_server(self):
        assert not _build_book("plantuml_server.toml", output_dir="plantuml_server_fail")
