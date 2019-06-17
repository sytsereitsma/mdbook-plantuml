@set __MDBOOK_PLANTUML_E2E_CD__=%cd%

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

@cd %~dp0\test_book
@mdbook build

:END
@cd %__MDBOOK_PLANTUML_E2E_CD__%
@set __MDBOOK_PLANTUML_E2E_CD__=
