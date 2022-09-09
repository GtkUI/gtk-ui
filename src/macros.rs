#![macro_use]

macro_rules! start_name_range{() => {'a'..='z' | 'A'..='Z' | '_'}}
macro_rules! name_range{() => {'a'..='z' | 'A'..='Z' | '-' | '_'}}
