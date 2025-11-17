#!/usr/bin/env bash

java \
  -Dsbe.target.language=Rust \
  -jar sbe-all-1.36.2.jar \
  ledger-schema.xml
#   -Dsbe.output.dir=sbe-codec \