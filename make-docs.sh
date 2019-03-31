#!/bin/env sh
asciidoctor -o docs/index.html Readme.adoc
sed -i 's/src="docs\//src="/' docs/index.html
