
# DevCamp8: Game of commons

This is the official repository for the [Holochain](https://www.holochain.org/) DevCamp 8 -- a community organized learning event for developers who want to develop on Holochain.
Here we present a step-by-step Holochain based implementation of the classical economics game [Tragedy of commons](https://en.wikipedia.org/wiki/Tragedy_of_the_commons). The word "tragedy" is removed from the name of our version for a more positive spin :)

## Repo structure

This repo has a branch for every DevCamp session about a specific part of this application. Every branch contains several commits to explain the step-by-step process of development, and the code compiles for every commit (if it doesn't, please open an issue pointing to a commit!).
If you're looking at this repository after the DevCamp, we recommend you follow the branches for every session to understand the process. There are a few things in the comments that are explained at the first occurence of any code, so reading code from earlier sessions would guarantee you bump into those comments early.

There's a separate file in the root of this repo called HOMEWORK.md. Homework isn't the best word, but it kinda fits: this file contains our recommendations on things to do with the code to make your learning more interactive.

## Context

We decided to implement a game for a few reasons:
1. (almost) no data input required. There are a lot of decentralized app examples that copy the existing platforms, such as twitter/youtube and so on, but all of them require users to actually start producing the data (which is often made up) to get familiar with the decentralized UX. Here, you just need to try and play the game.
2. it's collaborative. One of the main points of all decentralized apps is to enable/improve collaboration, so it seems logical to choose collaborative use-case for a decentralized framework intro. It also allows us to easily demostrate all the major implications of the Holochain's eventual consistency.

## Environment Setup

1. Install the holochain dev environment (only nix-shell is required): https://developer.holochain.org/docs/install/
2. Enable Holochain cachix with:

```bash
nix-env -iA cachix -f https://cachix.org/api/v1/install
cachix use holochain-ci
```

3. Clone this repo and `cd` inside of it.
4. Enter the nix shell by running this in the root folder of the repository: 

```bash
nix-shell
npm install
```

This will install all the needed dependencies in your local environment, including `holochain`, `hc` and `npm`.

## Building the DNA

- Build the DNA (assumes you are still in the nix shell for correct rust/cargo versions from step above):

```bash
npm run build:happ
```

## Running the DNA tests

```bash
npm run test
```

## UI

To test out the UI:

``` bash
npm start
```

To run another agent, open another terminal, and execute again:

```bash
npm start
```

Each new agent that you create this way will get assigned its own port and get connected to the other agents.

## Package

To package the web happ:

``` bash
npm run package
```

You'll have the `game-of-commons.webhapp` in `workdir`. This is what you should distribute so that the Holochain Launcher can install it.

You will also have its subcomponent `game-of-commons.happ` in the same folder`.

## Documentation

We are using this tooling:

- [NPM Workspaces](https://docs.npmjs.com/cli/v7/using-npm/workspaces/): npm v7's built-in monorepo capabilities.
- [hc](https://github.com/holochain/holochain/tree/develop/crates/hc): Holochain CLI to easily manage Holochain development instances.
- [@holochain/tryorama](https://www.npmjs.com/package/@holochain/tryorama): test framework.
- [@holochain/conductor-api](https://www.npmjs.com/package/@holochain/conductor-api): client library to connect to Holochain from the UI.
