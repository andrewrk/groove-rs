# groove-rs [![Build Status](https://travis-ci.org/andrewrk/groove-rs.svg?branch=master)](https://travis-ci.org/andrewrk/groove-rs)

Rust bindings to [libgroove](https://github.com/andrewrk/libgroove) -
streaming audio processing library.

[Documentation](https://s3.amazonaws.com/superjoe/doc/rust-groove/groove/index.html)

## Features

 * Safe interface - no functions are `unsafe`
 * Resources are automatically cleaned up for you.

## What's Done

 * opening files and adding to a playlist
 * basic raw sink support
 * basic endoder sink support

## What's Left to Do

 * miscellaneous API functions
 * groove-player API
 * groove-loudness-detector API
 * groove-fingerprinter API
