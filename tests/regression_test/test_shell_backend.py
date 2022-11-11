import unittest
import os
import base64
import hashlib
import shutil
import json
from parameterized import parameterized, parameterized_class
from preprocessor_runner import *
from file_locations import get_plantuml_calls_file, get_test_output_dir
import markdown_snippets
import preprocessor_builder

@parameterized_class(('piped', 'data_uri'), [
   (False, False),
   (True, True),
])
class TestShellBackend(unittest.TestCase):

    """
    This tester first builds the shell only version of mdbook-plantuml and then
    calls the executable directly, but with a fake plantuml command
    (python fake_plantuml.py). This fake command simply outputs the invocation
    arguments to a file (test_output/plantuml_calls.txt) so we can check it was
    called with the correct arguments.
    """
    @classmethod
    def setUpClass(cls):
        # make sure python is on the path. It is used to fake the plantuml
        # application to capture the arguments it is called with (and fake some
        # output)
        assert shutil.which("python") is not None, "Python cannot be found on the path"

        # Build mdbook-plantuml with shell support
        assert preprocessor_builder.build_shell()

    def setUp(self):
        if os.path.isdir(get_test_output_dir()):
            shutil.rmtree(get_test_output_dir())
        os.mkdir(get_test_output_dir())

        if os.path.exists(get_plantuml_calls_file()):
            os.remove(get_plantuml_calls_file())

        self.runner = PreprocessorRunner()
        fake_plantuml = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fake_plantuml.py")
        self.runner.set_preprocessor_config({
            "plantuml-cmd": f"python {fake_plantuml}",
            "piped": self.piped,
            "use-data-uris": self.data_uri,
        })

    def get_plantuml_calls(self):
        assert os.path.exists(get_plantuml_calls_file())
        return json.load(open(get_plantuml_calls_file(), "rt"))

    def __get_image_filename(self, call):
        name = hashlib.sha1(call["plantuml-code"].encode()).hexdigest()

        if "-tsvg" in call["arguments"]:
            filename = name + ".svg"
        else:
            filename = name + ".png"

        return filename

    def __get_image_path(self, call):
        filename = self.__get_image_filename(call)

        if "-pipe" in call["arguments"]:
            filename = os.path.join(get_test_output_dir(), ".mdbook-plantuml-cache", filename)
        else:
            filename = os.path.join(get_test_output_dir(), "mdbook-plantuml-img", filename)

        return filename        

    def assertFileIsCreated(self, call):
        img_path = self.__get_image_path(call)
        assert os.path.isfile(img_path)

    def format_md_link(self, call, prefix=""):
        if self.data_uri:
            if "-tsvg" in call["arguments"]:
                img_type = "svg+xml"
            else:
                img_type = "png"

            img_path = self.__get_image_path(call)
            img_data = open(img_path).read()            
            b64_data = base64.b64encode(img_data.encode()).decode()

            return f"data:image/{img_type};base64," + b64_data
        else:
            filename = self.__get_image_filename(call)
            url = prefix + "mdbook-plantuml-img/" + filename
            return "![]({url})".format(url=url)

    @parameterized.expand([
        (markdown_snippets.ab_class_diagram, "svg"),
        (markdown_snippets.ditaa, "png"),
    ])
    def test_plantuml_invocation(self, snippet, expected_ext):
        self.runner.set_content(Chapter("Chapter 1", snippet.markdown))

        result = self.runner.run()

        calls = self.get_plantuml_calls()
        self.assertEqual(1, len(calls))
        call = calls[0]
        self.assertIn("-t" + expected_ext, call["arguments"])
        self.assertFileIsCreated(call)
        self.assertEqual(snippet.plantuml_code, call["plantuml-code"])

        # Finally check if the correct link is in the chapter data
        self.assertIn(self.format_md_link(call),
                      result.root_chapter["content"])

    @parameterized.expand([
        (markdown_snippets.utxt_format, "utxt"),
    ])
    def test_plantuml_format_invocation(self, snippet, expected_ext):
        self.runner.set_content(Chapter("Chapter 1", snippet.markdown))

        result = self.runner.run()

        calls = self.get_plantuml_calls()
        self.assertEqual(1, len(calls))
        call = calls[0]
        self.assertIn("-t" + expected_ext, call["arguments"])
        self.assertEqual(snippet.plantuml_code, call["plantuml-code"])

    def test_nested_chapters(self):
        root_chapter = Chapter(
            "Chapter 1", markdown_snippets.ab_class_diagram.markdown)
        sub_chapter = Chapter(
            "Nested 1", markdown_snippets.cd_class_diagram.markdown)
        root_chapter.sub_items.append(sub_chapter)

        self.runner.set_content(root_chapter)
        result = self.runner.run()

        calls = self.get_plantuml_calls()
        self.assertEqual(2, len(calls))

        # Note that the calling order might change when the mdBook library changes
        self.assertIn(self.format_md_link(calls[1]),
                      result.root_chapter["content"])
        self.assertIn(self.format_md_link(calls[0], "../"),
                      result.nested_chapter["content"])

    def test_mathjax_is_untouched(self):
        mathjax = r"\\( \int x dx = \frac{x^2}{2} + C \\)"
        self.runner.set_content(Chapter("Chapter 1", mathjax))

        result = self.runner.run()

        self.assertIn(mathjax, result.root_chapter["content"])
