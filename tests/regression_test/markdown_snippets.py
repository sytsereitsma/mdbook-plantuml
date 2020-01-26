
class Snippet:
    def __init__(self, code):
        self.plantuml_code = code.strip()
        self.markdown = "```plantuml\n{}\n```".format(self.plantuml_code)


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
