# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Spritedit — a sprite editor that works in the browser. Built with Rust + Webassembly + WebGPU.

## Features

- Grid view that shows isometric grid for editing sprites.
- Can load sprites from an image, either via an URL, or upload.
- Save sprites to disk
- Set the sprite resolution, i.e. how many pixels in the sprite per grid box.
- Can paint the sprite, pixel by pixel, with transparency.
- Use GenAI to make new sprites
- Layout similar to VSCode:
  - A command panel at the top that opens with Cmd + Shift + P.


## App design
- UI is done all in rust, using imgui
- Minimal communication between javascipt and rust. Do as much as possible within rust.

