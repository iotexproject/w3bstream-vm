#!/bin/bash

echo start

echo y | clinic doctor --open=false -- node src/server.js && clinic doctor --open=false -- node src/server.js