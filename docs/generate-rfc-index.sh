#!/usr/bin/env bash

set -e

printf -- '- [RFCs](rfc/0001-rfc-process.md)\n\n' >> src/SUMMARY.md

find ./src/rfc ! -type d -name '*.md' -print0 \
  | sort -z \
  | while read -r -d '' file;
do
    printf -- '  - [%s](rfc/%s)\n' "$(basename "$file" ".md")" "$(basename "$file")"
done >> src/SUMMARY.md

