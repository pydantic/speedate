# not8601

[![CI](https://github.com/samuelcolvin/not8601/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/samuelcolvin/not8601/actions/workflows/ci.yml?query=branch%3Amain)
[![Coverage](https://codecov.io/gh/samuelcolvin/speedate/branch/main/graph/badge.svg?token=xCXg5aV9wM)](https://codecov.io/gh/samuelcolvin/speedate)

RFC 3339 & Common ISO 8601 date-time parsing.

Not iso8601 because iso8601 has lots of crazy formats - who thinks `2022-144T22` is a sensible datetime format?
See [https://ijmacd.github.io/rfc3339-iso8601/](https://ijmacd.github.io/rfc3339-iso8601/) for more info.

This will be the datetime parsing logic for [pydantic-core](https://github.com/samuelcolvin/pydantic-core).
