#!/bin/bash
export LD_LIBRARY_PATH=/home/morten/Dev/structure/target/debug:$LD_LIBRARY_PATH
cd application_core && cargo build && cd ../frontend && flutter run -d linux