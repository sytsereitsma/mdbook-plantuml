set ORGPATH=%PATH%
set ORGCD=%CD%

set PATH=%~dp0\..\..\target\release;%PATH%
cd %~dp0\..\..
cargo build --release
cd %ORGCD%

mdbook %*

set PATH=%ORGPATH%