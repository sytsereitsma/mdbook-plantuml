# Chapter 1

## Some text
```python
def a():
    pass
```

## Some text
```plantuml
@startuml
A --|> B
@enduml
```

## Some more text
```plantuml
@startuml
PlantUML --|> Server
@enduml
```

## Error
The diagram source below is not valid PlantUML syntax

```plantuml
E --|>
```

## And some ditaa
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

And graphviz diagrams
```plantuml
@startdot
digraph foo {
  node [style=rounded]
  node1 [shape=box]
  node2 [fillcolor=yellow, style="rounded,filled", shape=diamond]
  node3 [shape=record, label="{ a | b | c }"]

  node1 -> node2 -> node3
}
@enddot
```