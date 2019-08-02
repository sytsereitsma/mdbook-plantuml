# Chapter 1

Some text

```plantuml
@startuml
A --|> B
@enduml
```

Some more text

```plantuml
@startuml
C --|> D
@enduml
```

Error, forgot @startuml @enduml block

```plantuml
C --|> D
```

And some ditaa
```plantuml
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
```