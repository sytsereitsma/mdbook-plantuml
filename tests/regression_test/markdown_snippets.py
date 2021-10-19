
class Snippet:
    def __init__(self, code, extra=""):
        self.plantuml_code = code.strip()
        if extra:
            extra = "," + extra
        self.markdown = "```plantuml{}\n{}\n```".format(extra, self.plantuml_code)


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
