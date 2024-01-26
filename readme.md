# Nickkerish <!-- omit in toc -->

A dummy Jupyter Kernel implemented in Rust using ZeroMQ

- [1. Introduction](#1-introduction)
- [2. Acknowledgements](#2-acknowledgements)
  - [2.1. Copy pasted Doc-strings](#21-copy-pasted-doc-strings)
  - [2.2. Evcxr](#22-evcxr)
- [3. Usage](#3-usage)
  - [3.1. Show CLI Help](#31-show-cli-help)
  - [3.2. Install the kernelspec](#32-install-the-kernelspec)
- [4. Nick's Notes](#4-nicks-notes)
  - [4.1. Key Documentation Pages](#41-key-documentation-pages)
  - [4.2. Sockets](#42-sockets)
    - [4.2.1. `shell` Router](#421-shell-router)
    - [4.2.2. `iopub` Pub](#422-iopub-pub)
    - [4.2.3. `stdin` Router](#423-stdin-router)
    - [4.2.4. `control` Router](#424-control-router)
    - [4.2.5. `heartbeat` Rep](#425-heartbeat-rep)
  - [4.3. Identities](#43-identities)
  - [4.4. The Screwy Behavior of Various Clients](#44-the-screwy-behavior-of-various-clients)
    - [4.4.1. VS Code](#441-vs-code)
    - [4.4.2. Jupyter Lab](#442-jupyter-lab)
  - [4.5. The Screwy Behavior of Various Kernels](#45-the-screwy-behavior-of-various-kernels)
    - [4.5.1. Evcxr](#451-evcxr)
- [5. Other Notes and Shell Snippets](#5-other-notes-and-shell-snippets)

## 1. Introduction

I made this to explore what it takes to get a jupyter kernel working in rust.

It pretends to offer a non-existent language called `Nickkerish` which just
echos back any execution requests.

If this project turns out well, this might serve as a nice reference
implementation / template other rust projects could use to wrap other rust-based
programming languages. (e.g. maybe [Uiua](https://www.uiua.org/))

## 2. Acknowledgements

### 2.1. Copy pasted Doc-strings

Many (but not all) of the doc-strings in this project are copy pasted verbatim from
the [jupyter-client.readthedocs.io](https://jupyter-client.readthedocs.io/en/latest/index.html)
page which is under a
[BSD 3-Clause "New" or "Revised" License](https://github.com/jupyter/jupyter_client/blob/396e665af9088f4f083c02c12ea1fb4e9b3dff91/LICENSE).
My thanks to the original authors.

### 2.2. Evcxr

Although this crate is my own special kind of mess, I got started by reading
from the [evcxr](https://github.com/evcxr/evcxr) project. Currently this is
still a much better implementation than what I came up with here.

## 3. Usage

### 3.1. Show CLI Help

Build and run using cargo

```shell
cargo run -- help
```

### 3.2. Install the kernelspec

Install the kernelspec so that jupyter can find the kernel executable. Running
it via cargo means you are pointing the kernelspec at the development build; the
exe somewhere in the target directory:

```shell
cargo run -- install-kernel-spec
```

Run the kernel (normally you would not do this manually, this is called by your
jupyter front-end such as vscode or jupyter labs etc):

```shell
nickkerish.exe --connection-file "path/to/connection/file.json"
```

## 4. Nick's Notes

### 4.1. Key Documentation Pages

- [Handling messages](https://jupyter-client.readthedocs.io/en/latest/kernels.html#handling-messages)
  overview of endpoint functions
- [Kernel Specs](https://jupyter-client.readthedocs.io/en/latest/kernels.html#kernel-specs)
  A json kernel descriptor file; Jupyter can be made aware of your new kernel
  using the ```jupyter kernelspec install``` command.
- [Connection Files](https://jupyter-client.readthedocs.io/en/latest/kernels.html#connection-files)
  are provided via the command line be jupyter clients at startup to provide port numbers and ip
  addresses that the kernel is expected to create sockets on, and the key to be used for message
  verification.
  - Note: clients kinda ignore the language specified in this anyway, and its all redundant because
    the `kernel_info_response` message has better more complete information anyway!
- [Compatibility](https://jupyter-client.readthedocs.io/en/latest/messaging.html#compatibility)
  describes the minimum features required to produce a working kernel (very important for my lazy
  fingers 😊)

### 4.2. Sockets

The sockets are initially baffling to understand, my notes below are based on
[Messaging in Jupyter](https://jupyter-client.readthedocs.io/en/latest/messaging.html)

The socket types (Router, Pub, Rep, etc) are not that important to understand,
presumably they refer to different queuing and broadcast mechanisms. Some of
them can only send, some can only receive, some can do both, some broadcast to
all clients etc.

#### 4.2.1. `shell` Router

Most stuff happens over this socket

- `kernel_info_request` client requests details about version and language and
  some capabilities/settings
- `kernel_info_reply` kernel responds to client
- `execute_request` client sends code to kernel as string
- `execute_reply` kernel acknowledges execution request, but does not return the
  result yet (see `execute_result` below)
- `history_request`/`history_reply` can be ignored, but are required if multiple
  clients need to connect to the kernel and see the same thing.
- `is_complete_request` and `is_complete_reply` are used in a terminal
  environment to allow multi-line input. For example if the user opens a block
  and then hits return; the terminal will create an indented new line instead of
  submitting the command for execution.

#### 4.2.2. `iopub` Pub

Broadcasts messages to all clients
The critical messages are:

- `status` the Starting / Busy / Idle status of the kernel must be kept up to
  date before and after each request (which is kinda dumb, what is the point of
  an asynchronous queue protocol if you are just going to require synchronous
  behavior like that)
- `execute_result` returns the results of `execute_requests`

#### 4.2.3. `stdin` Router

Allows the kernel to send requests to the client for text/keyboard input which
is typically piped to stdin.

#### 4.2.4. `control` Router

Serves the same purpose as shell, but separated into another channel so that
critical messages are not queued being long running execution requests being
handled over the shell socket

The critical messages are:

- `shutdown_request`/`shutdown_reply` allows the client to request either a
  shutdown or a restart, and the kernel can acknowledge the request before doing
  it.
- `interrupt_request`/`interrupt_reply` allows the client to forward operating
  system interrupt signals if the kernel cannot catch them itself. Note that
  these are only used if the kernel specified `"interrupt_mode":"message"` in
  the 'kernel spec'

#### 4.2.5. `heartbeat` Rep

Kernel muse echo back immediately when receiving a message on this channel.
Typically the message received will be a single frame containing `b"ping"`.

### 4.3. Identities

The first frame(s) of a ZMQMessage before the delimiter `b"<IDS|MSG>"` are
called Identities. They are used by ZMQ for message routing. They must be cloned
onto any response messages.

For the `iopub` socket this is just a single frame containing the message
`topic`. By convention the `topic` is just a clone of the
`header.message_type`

> Some implementations may append additional information to the `topic`;e.g.
> `b"kernel.{u-u-i-d}.execute_result"` or `b"stream.stdout"` etc. Generally
> clients just subscribe to all topics, so the specific value may not be
> important.

### 4.4. The Screwy Behavior of Various Clients

#### 4.4.1. VS Code

- Does not accept unknown languages.
  - It will repeatedly send `kernel_info_request` messages if it cannot find a
    matching supported language.
  - So far the only way i have found to make it behave is to lie and tell it
    that I am a python kernel.
- Injects invisible code
  - Did you know that VS Code does a bunch of imports into your python while the
    kernel is starting?
  - Smells like telemetry... I mean, we know it is riddled with telemetry
    anyway.
  - Appears to be fairly harmless... looks like it injects some environment
    variables and captures the version of ipywidgets.

#### 4.4.2. Jupyter Lab

- Repeatedly requests `kernel_info_request` even if it promptly receives a valid result
  - Might be some kind of pre-flight / initialization process

### 4.5. The Screwy Behavior of Various Kernels

#### 4.5.1. Evcxr

- starts by broadcasting `kernel_status` of `"busy"` instead of `"starting"`

## 5. Other Notes and Shell Snippets

```bash
jupyter kernelspec list
```
