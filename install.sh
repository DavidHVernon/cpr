#!/bin/bash
set -x
cd /usr/local/bin
curl https://raw.githubusercontent.com/DavidHVernon/cpr/master/release/0.1.0/cpr.zip --output cpr.zip
unzip cpr.zip
rm cpr.zip 
