#!/bin/sh

cargo public-api -sss | rg -o '([a-zA-Z_]\w*)(::\w+)+' | rg -v '^std::' | rg -v '^alloc::' | rg -v '^core::' | rg -v '^Self::' | rg -v '^shinden_to_anilist_core::' | sort -u > leaked.txt