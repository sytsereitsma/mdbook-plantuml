## 0.4.0 (2019-09-25)
* PlantUML server support, woot!
* Fixed issue where all markdown after the first PlantUML image was not
  rendered anymore.

## 0.3.0 (2019-08-29)
* Generate SVG image files rather than inline svg
* For ditaa images revert to png, because PlantUML does not support svg for
  ditaa diagrams (issue #9)

## 0.2.0 (2019-07-08)
* Generated SVG image is now wrapped in a div with class type 'plantuml'
* More detailed error information when SVG generation failed (including a hint for a possible cause).

## 0.1.0 (2019-07-08)
* First version
