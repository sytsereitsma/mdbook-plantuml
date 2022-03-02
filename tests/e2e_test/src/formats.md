# Output formats

## Text output

```plantuml,format=txt
@startuml
C --> D
@enduml
```

## UTF-8 Text output
Only works on shell, web server renders ```txt``` format as utf8 and fails for utf8 format (strangely enough)

```plantuml,format=utxt
@startuml
C --> D
@enduml
```

## PNG output

```plantuml,format=png
@startuml
C --|> D
@enduml
```

## Braille output
Only works on shell

```plantuml,format=braille
@startuml
C --|> D
@enduml
```

## Unknown output error

```plantuml,format=chuck
@startuml
C --|> D
@enduml
```
