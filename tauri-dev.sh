#!/bin/bash
cd "$(dirname "$0")"
PATH="./.cargo/bin:$PATH" cargo tauri dev