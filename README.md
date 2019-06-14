# mdBook PlantUML

mdBook preprocessor to render [PlantUML](http://plantuml.com/) diagrams to images in the book output directory.

## Usage

First create the preprocessor in your book.toml file:
```toml
[book]
authors = ["Dzjengis Khan"]
multilingual = false
src = "src"
title = "mdBook PlantUML preprocessor"

[output.plantuml]
plantuml-cmd="java -jar plantuml.jar"

[[output.plantuml.sources]]
src=["../uml1/*.puml"]
output-dir="class-diagrams"

[[output.plantuml.sources]]
src=["../uml2/*.puml", "/uml3/*.puml"]
output-dir="state-diagrams"
```

## Options
- **plantuml-cmd:** Optional command override for PlantUML (defaults to "java -jar plantuml.jar").
- **sources:** An array of PlantUML sources and where to write the png images to.

Available configuration options for the `[[output.plantuml.sources]]` array:

- **src:** A list of source files, directories or globs you'd normally pass to PlantUML.
- **output-dir:** Output directory of the image files relative to the book output directory.

For the example config above the PlantUML executable would be called twice (assuming the book output dir is /foo/bar):

On windows (linux is the same, only with ```sh -c``` instead of ```cmd /C```)
```bat
cmd /C java -jar plantuml.jar -o \foo\bar\class-diagrams ..\uml1\*.puml
cmd /C java -jar plantuml.jar -o \foo\bar\state-diagrams ..\uml2\*.puml \uml3\*.puml
```

I initially used the [mdBook linkcheck](https://github.com/Michael-F-Bryan/mdbook-linkcheck) preprocessor as template for this project.