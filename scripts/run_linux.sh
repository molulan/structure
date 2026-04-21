#!/bin/bash
export LD_LIBRARY_PATH=/home/morten/Dev/Projects/structure/target/debug:$LD_LIBRARY_PATH
cd ../backend && cargo build && cd ../frontend && flutter run -d linux