# mdBook PlantUML

mdBook preprocessor to render [PlantUML](http://plantuml.com/) code blocks as images in your book.

## Contents
- [Usage](#usage)
- [Configuration options](#options)
- [Server configuration](#example-server-configuration)
- [Troubleshooting rendering issues](#troubleshooting-rendering-issues)
- [Change log](#change-log)
- [Building/installing on Linux](#building-and-installing-on-linux)

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

## Image formats
The image is svg, or png by default, depending on the diagram type. If desired it can be changed to another one of PlantUMLs output formats (note that some formats are not supported by all browsers and or PlantUML server implementations).

See https://plantuml.com/command-line (Types of Output File) for available formats. mdbook-plantuml uses the short param name (case sensitive, without the '-')

````markdown
A diagram in UTF-8 text format (inlined automatically)
```plantuml,format=utxt
@startuml
A --|> B
@enduml
```

Force png format:
```plantuml,format=png
@startuml
A --|> B
@enduml
```
````

## Options
- **plantuml-cmd:** Optional command override for PlantUML (by default it is attempted to auto detect "java -jar plantuml.jar" or "plantuml").
  When a URL is provided it is assumed you want to generate the images using a PlantUML server implementation.
- **clickable-img:** Optional (`false` by default). When `true`, images can be clicked and are opened in a new tab/window.
- **use-data-uris:** Optional (`true` by default). When `true`, images are rendered as inline Data URIs (not requiring image files in the src dir).
- **piped:** Optional (`true` by default). When `true`, images are rendered using PlantUML in piped mode (i.e. not needing temporary files), meaning the `!include` directive family can be used. When `false` 'legacy' mode is used, meaning temporary files are generated for image generation.
- **verbose:** Optional (`false` by default). When `true`, debug logging is enabled.

## Features
- **plantuml-server** Add http server support only
- **plantuml-ssl-server** Add https server support (default)

Examples:
Install without server support:
```cargo install mdbook-plantuml --no-default-features```

Install with http server support:
```cargo install mdbook-plantuml --no-default-features --features plantuml-server```

Install with https server support:
```cargo install mdbook-plantuml --no-default-features --features plantuml-ssl-server```

## Example Data URI configuration
```toml
[book]
authors = ["Chuck Norris"]
multilingual = false
src = "src"
title = "mdBook PlantUML preprocessor"

[preprocessor.plantuml]
plantuml-cmd="plantuml"
use-data-uris=true
```

## Example server configuration

Below is an example server configuration.

You can test your server by appending the URL with "/png/SoWkIImgAStDuGh8ISmh2VNrKT3LhR5J24ujAaijud98pKi1IW80".
Using the example below this example you'd end up with [this URL](http://localhost:8080/plantuml/png/SoWkIImgAStDuGh8ISmh2VNrKT3LhR5J24ujAaijud98pKi1IW80).
When it is working correctly you should see the following image:

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

## Troubleshooting rendering issues
mdBook communicates to the preprocessor using stdio. As a result log output
from the preprocessor is not printed to the screen. When the preprocessor's
markdown error output is insufficient for you it is possible to redirect 
logging to the file ./output.log by using the command line switch -l. See the
config below for an example:

```toml
[book]
authors = ["Sytse Reitsma"]
multilingual = false
src = "src"
title = "mdBook E2E test book"

[preprocessor.plantuml]
plantuml-cmd="http://localhost:8080/plantuml"
command = "mdbook-plantuml -l"
```

## Change log

### 0.9.0 (2022-11-11)
* Use PlantUML in piped mode by default, meaning the PlantUML `!include` directives can be used (thanks @albenik)
  * Enabled by default, see the `piped` config option to disable piped mode
* Auto detect PlantUML executable from `java -jar plantuml.jar` or `plantuml`
* Fail when PlantUML cannot be found or is improperly configured (e.g. configure a server backend when the server feature is disabled)
* Better feedback of the rendering process by logging to stderr (thanks @
mini-ninja-64 for the idea)
* Data URIs are now the default rendering mode (see use-data-uris in book config option)

### 0.8.0 (2022-07-04)
* Many thanks to @danieleades for cleanup and modernization
* Click on an image to open it in isolation (credit @YushiOMOTE)
* Add support for `puml` code blocks as an alternative to plantuml (native IDE support for various programs, credit @YushiOMOTE)
* Diagram formats can now be configured per code block (e.g. png, or svg)
* Support for [Data URLs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs), credit @ knightflower1989). This feature can be used to workaround the mdbook serve loop issue.

### 0.7.0 (2020-08-29)
* 🏎️ Speed! Added caching to only regenerate the changed code blocks, instead of all of them.
* Feature gated the PlantUML server and ssl server (default is ssl server).
  [Issue #16](https://github.com/sytsereitsma/mdbook-plantuml/issues/16)
* Fixed infinite rebuild loop when using the ```mdbook serve``` command.
  Because the preprocessor output cannot be written directly to the book output
  dir anymore the images need to be created in the src dir unfortunately (mdBook
  change). You still end up with one extra rebuild when images are updated, I cannot
  prevent this (the gitignore file of mdbook should be able to prevent this,
  but it does not).
  [Issue #17](https://github.com/sytsereitsma/mdbook-plantuml/issues/17)
  
### 0.6.0-beta (2020-01-29)
* 🏎️ Speed! Added caching to only regenerate the changed code blocks, instead of all of them.
* Feature gated the PlantUML server and ssl server (default is ssl server).
  [Issue #16](https://github.com/sytsereitsma/mdbook-plantuml/issues/16)
* Dropped pulldown-cmark in favor of a home grown markdown parser. The conversion
  from markdown and back caused changes in the document.
  [Issue #15](https://github.com/sytsereitsma/mdbook-plantuml/issues/15)
* These are pretty major changes, hence the beta label.

### 0.5.0 (2019-11-08)
* mdBook from v0.3.2 on deletes the book output directory when rendering starts,
  causing all generated preprocessor output to be deleted too. The only workaround
  at the moment is outputting the images to the src directory. This is ugly, but
  a temporary solution until mdBook allows for other ways of adding resources
  from a preprocessor.
  See [this issue](https://github.com/rust-lang/mdBook/issues/1087)
* More thorough README.md

### 0.4.0 (2019-09-25)
* PlantUML server support, woot!
* Fixed issue where all markdown after the first PlantUML image was not
  rendered anymore (preprocessors cannot output HTML apparently).
* Images in nested chapters now have the correct URL (thanks @rafasf).
* Allow logging to file to troubleshoot preprocessor issues.

### 0.3.0 (2019-08-29)
* Generate SVG image files rather than inline svg
* For ditaa images revert to png, because PlantUML does not support svg for
  ditaa diagrams (issue #9)

### 0.2.0 (2019-07-08)
* Generated SVG image is now wrapped in a div with class type 'plantuml'
* More detailed error information when SVG generation failed (including a hint for a possible cause).

### 0.1.0 (2019-07-08)
* First version

## Building and installing on Linux
I primarily work on Windows, and rust is not that portable yet apparently.

When you get the following error while installing/building mdbook-plantuml on Linux:
```
failed to run custom build command for `openssl-sys v0.9.49`
```

Make sure you have libssl-dev and pkg-config installed:
```sh
sudo apt update
sudo apt install libssl-dev pkg-config
```
