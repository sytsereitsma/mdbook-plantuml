# End 2 end tester
Runs mdbook with mdbook-plantuml to generate real books with some scenarios.
> You'll have to manually check if the books are ok (web pages are opened)

## Web server for test
Simply start a docker server
```
docker pull plantuml/plantuml-server
docker run -d -p 8080:8080 plantuml/plantuml-server:jetty
```

## Run the regression tests
See README.md one level up