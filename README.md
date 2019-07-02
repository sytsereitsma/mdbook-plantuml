# mdBook PlantUML

mdBook preprocessor to render [PlantUML](http://plantuml.com/) code blocks to embedded SVG in your book.

## Usage

First create the preprocessor in your book.toml file:
```toml
[book]
authors = ["Dzjengis Khan"]
multilingual = false
src = "src"
title = "mdBook PlantUML preprocessor"

[preprocessor.plantuml]
plantuml-cmd="plantuml.exe"
```

Then simply add a PlantUML code block in your book text:
````markdown
Some text here

```plantuml
@startuml
A --|>B
@enduml
```

Some more text.

````

The plantuml code block will be replaced with inline SVG data.

## Options
- **plantuml-cmd:** Optional command override for PlantUML (defaults to "java -jar plantuml.jar").
