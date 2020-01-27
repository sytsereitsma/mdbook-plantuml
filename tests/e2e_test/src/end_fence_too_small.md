## Start fence larger than end fence
Only on the server this causes a rendering error because of the extra tildes
after @enduml

~~~~~plantuml
@startuml
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response

Alice -> Bob: Another authentication Request
Alice <-- Bob: Another authentication Response
@enduml
~~~