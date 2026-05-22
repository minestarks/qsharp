# Quantum Development Kit Extension Talk

## 1. Outline

Introduction slide: introduce yourself, set expectations, and mention the prize challenge.

Introduction: We’ll walk through the Quantum Development Kit extension in Visual Studio Code and show how to get started.

Installation: VS Code, QDK extension, GitHub account, and setup check.

QDK features walkthrough: Open, run, visualize, and inspect quantum code.

Quantum Katas walkthrough: Start the first kata and show how to continue.

Challenge: Complete the first Quantum Kata, show a Bell state histogram, or make any circuit diagram for a chance to win a laptop.

---

## 2. Full Talk Draft

### Introduction

Hi, I’m Mine Starks. I’m a principal software engineer on the QDK team.

Today we’re going to walk through the QDK extension in VS Code. We’ll install the tools, have Copilot write some quantum programs for us, play with them, and learn how things work as we go.

There will be a challenge with a prize at the end, so definitely follow along. If you did not bring a laptop, do not worry. You may still be able to participate with your phone or tablet.

This is not a deep theory session. We’ll keep the theory light and spend most of the time in the tools. I also know this room may have a mix of backgrounds. Some of you may be totally new to quantum computing and just curious. Some of you may already work in quantum, but have not used the QDK yet. Both are totally fine. I just want you to leave knowing how to get started with the QDK and Copilot.

Before we jump in, quick show of hands: Raise your hand if you're totally new to quantum computing. Raise your hand if you've used other quantum development tools? And how many have already installed the QDK extension?

### Installation and setup

Show the installation instructions slide before switching into the demo.

We’ll start from the assumption that not everyone has everything installed yet. Some of you may already have Visual Studio Code, the QDK extension, and a GitHub account ready to go. If so, you can follow along, or help someone nearby.

For everyone else, we’ll walk through the setup together:

1. Open or install Visual Studio Code. [https://aka.ms/vscode ](https://aka.ms/vscode)
2. Install the Quantum Development Kit extension. (Search in Extension Manager)
3. Sign into or create a GitHub account if needed. 
4. Confirm that the extension is available and ready to use.

Once everyone is ready, we’ll move on and start poking at quantum code.

For the live setup demo, I’ll use a clean environment so you can see the install path from scratch. Then I’ll switch back to an environment where everything is already installed, so we are not depending on a live install behaving perfectly.

### QDK features walkthrough

This section needs to be fleshed out with the exact features to demo.

Possible feature walkthrough structure:

1. Show where the QDK extension appears in VS Code.
2. Open or create a simple quantum program.
3. Run the program locally.
4. Show results in the development environment.
5. Visualize output, such as a histogram.
6. Generate or view a circuit diagram.
7. Point out where examples, commands, documentation, or kata links live.

The key is to keep this section demo-first. Each feature should answer: “What can I click, run, or see right now?” Ideally without summoning any mysterious terminal goblins.

### Quantum Katas walkthrough

Next, we’ll move into the Quantum Katas. The katas are a way to practice quantum concepts through small, guided exercises.

We’ll show how to open the katas, start the first one, and understand what success looks like. The goal is not to finish everything live. It is to make sure you know how to keep going after this session, when no one is standing at the front of the room narrating your clicks.

This section connects directly to the challenge, since one of the challenge options is to complete the first Quantum Kata.

### Challenge: chance to win a laptop

At the end, we’ll present three possible challenges. Completing one gives you a chance to win a laptop. For anyone on a phone or tablet, there will be a mobile-friendly option: go to our website and complete the first Quantum Kata.

Challenge 1: Complete the first Quantum Kata.

Challenge 2: Show a Bell state histogram.

Challenge 3: Make any circuit diagram.

Mobile option: Go to our website and complete the first Quantum Kata.

Since the whole talk is happening on one tiny screen, the challenge slide should stay very simple. Instead of one crowded slide with all three challenges fighting for space, use three rotating slides, each focused on one challenge. Each slide should show the challenge title, the success condition, and one short hint.

Possible rotating challenge slides:

**Slide 1: Complete the first Quantum Kata**

- Success: show the completed first kata.
- Hint: start from the Quantum Katas section.

**Slide 2: Show a Bell state histogram**

- Success: run a Bell state example and show the histogram output.
- Hint: look for results close to 50/50 between `00` and `11`.

**Slide 3: Make any circuit diagram**

- Success: display any generated quantum circuit diagram.
- Hint: use a simple circuit if you are new.

---

## 3. Notes

### Dry run notes

Second run notes: have an introduction slide. Have a slide with installation instructions. For setup, use a VM, profile, or other clean environment for the install walkthrough, then switch back to the already-installed environment for the rest of the demo. Do not risk the whole talk on a live install.

### Screen and switching constraints

The entire thing will be on one tiny screen, so the talk needs careful transitions between slides and live demo.

Guiding rule: slides should give orientation; demos should carry the substance.

Use slides only when they reduce cognitive load:

- Title / intro
- Setup checklist
- Feature roadmap
- Kata transition
- Challenge instructions

Avoid dense slides, especially during the challenge. Use simple slides with large text.

### Demo choreography

Possible pattern:

1. Show a slide with the next step.
2. Switch to VS Code and perform that step.
3. Pause briefly to explain what changed.
4. Switch back only when the audience needs the next instruction.

### Challenge slide idea

Use three rotating challenge slides every few seconds instead of one crowded slide. This could be done as:

- Three separate slides with auto-advance timing.
- A looping PowerPoint or Keynote section.
- A browser-based mini slide loop if easier.

Each challenge slide should be readable from the back of the room and understandable in under five seconds.

### Tech notes to confirm

- Exact QDK extension name in the VS Code marketplace.
- Whether attendees need GitHub Codespaces, local VS Code, or both.
- Whether GitHub account creation is required or just recommended.
- Whether the demo depends on internet access.
- Whether the Bell state histogram is available directly in the current QDK workflow.
- Exact steps for generating a circuit diagram.
- Whether the laptop challenge requires verification, submission, or showing the result to a person live.
