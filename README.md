# templates

A custom template engine in development for [Ramon](https://github.com/reujab/ramon).

## Features

- Fully custom lexer and parser
- Mathematical operations with order of operations `{{ 1 + x * (1 + 2) }}`
- Minimal allocation
- Custom functions `Hello {{ world() }}`
- Logic `{{ if x }}x{{ elif y }}y{{ else }}neither{{/fi}}`
