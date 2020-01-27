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
@set PATH=%~dp0\..\..\target\release\;%PATH%

:RUN_TESTS
@cd %~dp0\..\..
@cargo build --release

@cd %~dp0

echo Test plantuml shell (outputs book to plantuml_shell directory)
@copy /Y plantuml_shell.toml book.toml
@IF EXIST rmdir /s /q .plantuml-cache
@mdbook build
@rmdir /s /q shell_book
@move book shell_book
@start shell_book\index.html

echo Test plantuml server (outputs book to plantuml_server directory)
@copy /Y plantuml_server.toml book.toml
@IF EXIST rmdir /s /q .plantuml-cache
@mdbook build
@rmdir /s /q server_book
@move book server_book
@start server_book\index.html

echo Test caching
@copy /Y plantuml_cache.toml book.toml
@IF EXIST rmdir /s /q .plantuml-cache
@mdbook build
@rmdir /s /q cache_book_1st
@move book cache_book_1st
@start cache_book_1st\index.html
echo Build again, the cache should be used now (how to test, other than observe the significantly smaller runtime?)
@mdbook build
@rmdir /s /q cache_book_2nd
@move book cache_book_2nd
@start cache_book_2nd\index.html


:END
@cd %__MDBOOK_PLANTUML_E2E_CD__%
@set __MDBOOK_PLANTUML_E2E_CD__=
