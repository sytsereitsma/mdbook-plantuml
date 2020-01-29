import unittest
import os
import shutil
import json
from parameterized import parameterized
from preprocessor_runner import *
from file_locations import get_shell_calls_file, get_test_output_dir
import markdown_snippets
import preprocessor_builder

class ShellBackendTester(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert preprocessor_builder.build_shell()

    def setUp(self):
        if os.path.isdir(get_test_output_dir()):
            shutil.rmtree(get_test_output_dir())
        os.mkdir(get_test_output_dir())

        if os.path.exists(get_shell_calls_file()):
            os.remove(get_shell_calls_file())

        self.runner = PreprocessorRunner()
        self.runner.set_preprocessor_config({"plantuml-cmd": "python shell_command.py"})

    def __get_shell_command_calls(self):
        assert os.path.exists(get_shell_calls_file())
        return json.load(open(get_shell_calls_file(), "rt"))

    def __get_image_filename(self, call):
        image_name = os.path.basename(call["arguments"][-1])
        name,ext = os.path.splitext(image_name)
        if "-tsvg" in call["arguments"]:
            filename = name + ".svg"
        else:
            filename = name + ".png"

        return filename

    def assertFileIsCreated(self, call):
        filename = self.__get_image_filename(call)
        assert os.path.isfile(
            os.path.join(get_test_output_dir (), "mdbook-plantuml-img", filename))

    def format_md_link(self, filename, prefix=""):
        url = prefix + "mdbook-plantuml-img/" + filename
        return "![{url}]({url})".format(url=url)

    @parameterized.expand([
        (markdown_snippets.ab_class_diagram, "svg"),
        (markdown_snippets.ditaa, "png"),
    ])
    def test_plantuml_invocation(self, snippet, expected_ext):
        self.runner.set_content(Chapter("Chapter 1", snippet.markdown))

        result = self.runner.run()
        self.assertEqual(0, result.returncode)

        calls = self.__get_shell_command_calls()
        self.assertEqual(1, len(calls))
        call = calls[0]
        self.assertIn("-t" + expected_ext, call["arguments"])
        self.assertFileIsCreated(call)
        self.assertEqual(snippet.plantuml_code, call["plantuml-code"])

        #Finally check if the correct link is in the chapter data
        filename = self.__get_image_filename(call)
        self.assertIn(self.format_md_link(filename), result.root_chapter["content"])

    def test_nested_chapters(self):
        root_chapter = Chapter("Chapter 1", markdown_snippets.ab_class_diagram.markdown)
        sub_chapter = Chapter("Nested 1", markdown_snippets.cd_class_diagram.markdown)
        root_chapter.sub_items.append(sub_chapter)

        self.runner.set_content(root_chapter)
        result = self.runner.run()

        calls = self.__get_shell_command_calls()
        self.assertEqual(2, len(calls))

        # Note that the calling order might change when the mdBook library changes
        root_filename = self.__get_image_filename(calls[1])
        nested_filename = self.__get_image_filename(calls[0])

        self.assertIn(self.format_md_link(root_filename), result.root_chapter["content"])
        self.assertIn(self.format_md_link(nested_filename, "../"), result.nested_chapter["content"])

    def test_mathjax_is_untouched(self):
        mathjax = r"\\( \int x dx = \frac{x^2}{2} + C \\)"
        self.runner.set_content(Chapter("Chapter 1", mathjax))

        result = self.runner.run()
        self.assertEqual(0, result.returncode)

        self.assertIn(mathjax, result.root_chapter["content"])



