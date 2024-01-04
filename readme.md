# Nickkerish <!-- omit in toc -->

A dummy Jupyter Kernel implemented in Rust using ZeroMQ

Made to explore building a jupyter kernel from scratch.

It pretends to offer a non-existent language called `Nickkerish`.

Currently it isn't working yet.

- [1. Sockets](#1-sockets)
  - [1.1. `shell` Router](#11-shell-router)
  - [1.2. `iopub` Pub](#12-iopub-pub)
  - [1.3. `stdin` Router](#13-stdin-router)
  - [1.4. `control` Router](#14-control-router)
  - [1.5. `heartbeat` Rep](#15-heartbeat-rep)
- [structure](#structure)
- [Dev Notes](#dev-notes)


## 1. Sockets

The sockets are baffeling to understand, my notes below

<https://jupyter-client.readthedocs.io/en/latest/messaging.html>

### 1.1. `shell` Router

### 1.2. `iopub` Pub

broadcast side effects

### 1.3. `stdin` Router

### 1.4. `control` Router

Serves the same purpose as shell, but separated into another channel so that
critical messages are not queued being long running execution requests being
handled over the shell socket

Used for shutdown, restart and debug messages.

### 1.5. `heartbeat` Rep

Kernel muse echo back immediately when receiving a message on this channel.

## structure


## Dev Notes

```bash
jupyter kernelspec list
```
