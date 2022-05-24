# not8601

[![CI](https://github.com/samuelcolvin/not8601/workflows/ci/badge.svg?event=push)](https://github.com/samuelcolvin/not8601/actions?query=event%3Apush+branch%3Amain+workflow%3Aci)

RFC 3339 & Common ISO 8601 date-time parsing.

Not iso8601 because iso8601 has lots of crazy formats - who thinks `2022-144T22` is a sensible datetime format?
See [https://ijmacd.github.io/rfc3339-iso8601/](https://ijmacd.github.io/rfc3339-iso8601/) for more info.

This will be the datetime parsing logic for [pydantic-core](https://github.com/samuelcolvin/pydantic-core).
