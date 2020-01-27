# Chapter 1

## MathJax
\\( \int x dx = \frac{x^2}{2} + C \\)

## A code block with another info string (python)
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
C --|> D
@enduml
```

## Error, forgot @startuml @enduml block (shell error only)

```plantuml
C --|> D
```

## Error in PlantUML diagram source

```plantuml
@startuml
C --|>
@enduml
```

## Ditaa support
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

## Graphviz diagram support
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

## More MathJax
To make sure we do not modify it further in the document.

\\( \int x dx = \frac{x^2}{2} + C \\)

## With tildes
~~~plantuml
@startuml
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response

Alice -> Bob: Another authentication Request
Alice <-- Bob: Another authentication Response
@enduml
~~~

## With 3 space indent and tildes
   ~~~plantuml
   @startuml
   Alice -> Bob: Authentication Request
   Bob --> Alice: Authentication Response
   
   Alice -> Bob: Another authentication Request
   Alice <-- Bob: Another authentication Response
   @enduml
   ~~~

## 4 space indent is considered unfenced code block (not rendered)
    ~~~plantuml
    @startuml
    Alice -> Bob: Authentication Request
    Bob --> Alice: Authentication Response
    
    Alice -> Bob: Another authentication Request
    Alice <-- Bob: Another authentication Response
    @enduml
    ~~~
