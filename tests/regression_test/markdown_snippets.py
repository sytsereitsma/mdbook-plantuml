
class Snippet:
    def __init__(self, code, extra=""):
        self.plantuml_code = code
        if extra:
            extra = "," + extra
        # Assert we can format a valid code block without
        assert self.plantuml_code[-1] == "\n" 
        self.markdown = "```plantuml{}\n{}```".format(extra, self.plantuml_code)


ab_class_diagram = Snippet("""\
@startuml
A --|> B
@enduml
""")

cd_class_diagram = Snippet("""\
@startuml
C --|> D
@enduml
""")


ditaa = Snippet("""\
@startditaa
+--------+   +-------+    +-------+
|        +---+ ditaa +--> |       |
|  Text  |   +-------+    |diagram|
|Document|   |!magic!|    |       |
|     {d}|   |       |    |       |
+---+----+   +-------+    +-------+
    :                         ^
    |       Lots of work      |
    +-------------------------+
@endditaa
""")

utxt_format = Snippet("""\
@startuml
C --|> D
@enduml
""", "format=utxt")
