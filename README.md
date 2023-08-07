# TestAngel

## Introduction

TestAngel makes automating tests easy by allowing you to make reusable actions that can be used as part of a bigger test process.

In TestAngel, you start off creating a Test Flow. This will be the instructions followed by TestAngel to complete your automation. This flow can then be built up of different actions, which can be provided from two sources. Actions can either come directly from engines, which can perform low-level tasks with systems, for example the HTTP engine can make HTTP requests. Alternatively, actions can come from custom-made complex actions. These are pre-defined flow-within-a-flows that allow complex, repetitive behaviour to be abstracted into it's own self-contained action.

## Parts

| Part | Description |
|:-----|:------------|
|`testangel`|The main executable and UI that controls the platform ("the controller").|
|`testangel-ipc`|The library that contains the serialisable messages that can be exchanged between the controller and the engine plugins.|
|`testangel-arithmetic`|An arithmetic engine plugin.|
