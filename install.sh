#!/bin/bash
set -x
cd /usr/local/bin
if test -f "cpr.zip"; then
    rm cpr.zip
fi
curl https://raw.githubusercontent.com/DavidHVernon/cpr/master/release/0.1.0/cpr.zip --output cpr.zip
if test -f "cpr"; then
    rm cpr
fi
unzip cpr.zip
rm cpr.zip 
