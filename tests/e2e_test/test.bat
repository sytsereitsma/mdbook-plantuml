@set __MDBOOK_PLANTUML_E2E_CD__=%cd%

@for /f "delims=" %%i in ('where mdbook-plantuml.exe 2^> NUL ^| findstr .cargo') do @(
    @echo mdbook-plantuml is installed in %%i, e2e test cannot run because mdbook ^
will not invoke the locally build preprocessor.
    @goto END
)

@if "%__MDBOOK_PLANTUML_E2E_ENV__%" == "" (
	@set __MDBOOK_PLANTUML_E2E_ENV__=1
) else (
    @echo Environment already set.
	@goto RUN_TESTS
)

:SETUP_ENV:
@set PATH=%~dp0\..\..\target\debug\;%PATH%

:RUN_TESTS
@cd %~dp0\..\..
@cargo build

@cd %~dp0

echo Test plantuml shell (output book to plantuml_shell directory)
@copy /Y plantuml_shell.toml book.toml
@mdbook build
@rmdir /s /q shell_book
@move book shell_book
@start shell_book\index.html

echo Test plantuml server (output book to plantuml_server directory)
@copy /Y plantuml_server.toml book.toml
@mdbook build
@rmdir /s /q server_book
@move book server_book
@start server_book\index.html

echo Test plantuml alternative command (output book to plantuml_alt directory)
del plantuml_alt.jar
mklink 
@copy /Y plantuml_server.toml book.toml
@mdbook build
@rmdir /s /q alt_book
@move book alt_book
@start alt_book\index.html

echo Test caching
del plantuml_alt.jar
mklink 
@copy /Y plantuml_cache.toml book.toml
@mdbook build
@rmdir /s /q cache_book
@move book cache_book
@start cache_book\index.html

:END
@cd %__MDBOOK_PLANTUML_E2E_CD__%
@set __MDBOOK_PLANTUML_E2E_CD__=
