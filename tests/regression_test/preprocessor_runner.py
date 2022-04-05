import json
import os
import subprocess
import logging


class Result:
    def __init__(self, stdout):
        self.__json = json.loads(stdout)

    @property
    def json(self):
        return self.__json

    @property
    def root_chapter(self):
        return self.json["sections"][0]["Chapter"]

    @property
    def nested_chapter(self):
        return self.root_chapter["sub_items"][0]["Chapter"]


class Chapter:
    def __init__(self, name, content):
        self.name = name
        self.content = content
        self.sub_items = []

    def to_dict(self, number=[1]):
        path = ""
        for _ in range(1, len(number)):
            path += "nested/"
        path += self.name + "_" + "-".join(map(str, number)) + ".md"

        data = {
          "name": self.name,
          "content": self.content,
          "number": number,
          "sub_items": [],
          "path": path,
          "parent_names": [] #TODO fill this when needed
        }

        for i, sub in enumerate(self.sub_items):
            sub_number = list(number)
            sub_number.append(i + 1)
            data["sub_items"].append (sub.to_dict(sub_number))

        return {"Chapter": data}


class PreprocessorRunner:
    def __init__(self):
        tester_root = os.path.dirname(__file__)
        book_filename = os.path.join(tester_root, "book.json")
        self.__book = json.load(open(book_filename, "rt"))

    def set_preprocessor_config(self, config):
        self.__book[0]["config"]["preprocessor"]["plantuml"] = config

    def set_content(self, chapter):
        self.__book[1]["sections"][0] = chapter.to_dict()

    def run(self):
        tester_root = os.path.dirname(__file__)
        preprocessor_cmd = os.path.join(tester_root, "..", "..", "target",
                                        "release", "mdbook-plantuml.exe")
        logging.info(f"Preprocessor cmd: '{preprocessor_cmd}'")
        logging.debug(f"Book: '{json.dumps(self.__book, indent=2)}'")

        proc = subprocess.Popen(
            [preprocessor_cmd],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        stdout, stderr = proc.communicate(json.dumps(self.__book).encode(),
                                          timeout=5.0)
        if proc.returncode != 0:
            logging.error(f"{preprocessor_cmd} stderr: {stderr}")
            raise subprocess.CalledProcessError(
                proc.returncode,
                preprocessor_cmd,
                stdout,
                stderr
            )

        return Result(stdout)
