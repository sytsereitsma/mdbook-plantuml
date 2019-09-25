# mdBook PlantUML

mdBook preprocessor to render [PlantUML](http://plantuml.com/) code blocks as images in your book.

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

The above assumes both the mdbook-preprocessor and the plantuml executable are on your path.

Then simply add a PlantUML code block in your book text:
````markdown
Some text here

```plantuml
@startuml
A --|> B
@enduml
```

Some more text.

````

The plantuml code block will be replaced an image reference to an SVG image if
possible, or png if PlantUML does not support svg for the requested diagram type
(i.e. ditaa).

## Options
- **plantuml-cmd:** Optional command override for PlantUML (defaults to "java -jar plantuml.jar" on Windows and "/usr/bin/plantuml" on Linux).
  When a URL is provided it is assumed you want to generate the images using a PlantUML server implementation.

## Example server configuration

Below is an example server configuration.

You can test your server by appending the URL with "/png/SoWkIImgAStDuGh8ISmh2VNrKT3LhR5J24ujAaijud98pKi1IW80", in this example you'd end up with [this URL](http://localhost:8080/plantuml/png/SoWkIImgAStDuGh8ISmh2VNrKT3LhR5J24ujAaijud98pKi1IW80). When it is working correctly you should see the following image:
![](doc/img/server_output.png)

```toml
[book]
authors = ["Dzjengis Khan"]
multilingual = false
src = "src"
title = "mdBook PlantUML preprocessor"

[preprocessor.plantuml]
plantuml-cmd="http://localhost:8080/plantuml"
```
