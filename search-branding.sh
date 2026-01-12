#!/bin/sh
grep -F 'Yaak' -r --exclude-dir={.git,.github,.husky}
