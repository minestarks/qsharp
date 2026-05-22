# Quantum Development Kit Workshop

**Audience:** Students new to quantum computing  
**Core message:** You can start exploring quantum computing right now with AI as your guide, even before you understand everything.  
**Duration:** ~35 minutes + 10 minute challenge

---

## 1. Introduction (~1 min)

Hi, I'm Mine Starks. I'm a principal software engineer on the Quantum Development Kit team at Microsoft.

:: todo - personal background, philosophy on dev tools, etc, maybe talk about typescript.

Today we're going to write and run a quantum program, visualize it, break it, fix it, and understand what it's doing — all using GitHub Copilot as our guide. You don't need any quantum computing background. You just need VS Code.

There will be a challenge with a prize at the end, so definitely follow along on your own laptop. If you didn't bring one, you may still be able to participate on your phone or tablet.

Quick show of hands: who here is totally new to quantum computing? Who's used other quantum tools before? Who already has the QDK extension installed?

This is not a deep theory session. We’ll keep the theory light and spend most of the time in the tools. I just want you to leave knowing how to get started with the QDK and Copilot.

:: todo - if you already know what you're doing, and the algorithms look too trivial to you that's not the point - you can apply to more complex things etc.

---

## 2. Installation Check (~5 min)

**[Slide: installation steps]**

Note: installation instructions should be sent out in advance so attendees can arrive ready. But assume some people will not have done it.

We’ll start from the assumption that not everyone has everything installed yet. Some of you may already have Visual Studio Code, the QDK extension, and a GitHub account ready to go. If so, you can follow along, or help someone nearby.

For everyone else, let's walk through the set up toether:

1. Open or install VS Code. [https://aka.ms/vscode](https://aka.ms/vscode)
2. Install the Quantum Development Kit extension — search for it in Extensions, look for the blue checkmark next to microsoft.com.
3. Sign into GitHub from VS Code (account icon, bottom-left).
4. Confirm: you should see the Quantum Möbius strip logo in the Activity Bar on the left. And you should be able to open Copilot Chat.

Everyone good? Raise a hand if you need help. Help your neighbor if they're stuck.

**[Wait. Don't rush this. Dead time here is fine.]**

---

## 3. Quantum Coin Flip — Follow Along (~12 min)

### Write It

**[Slide: the prompt to type]**

Everyone open Copilot Chat. Type this prompt — I'll leave it on screen:

> Write a Q# program that flips a quantum coin. Put a qubit in superposition and measure it.

Hit enter. You should get something back. It might look slightly different from mine — that's fine, Copilot is creative. As long as it puts a qubit in superposition with an H gate and then measures it, it's doing the right thing.

**[Do it live. Show the result.]**

Mine looks something like this:

```qsharp
operation Main() : Result {
    use q = Qubit();
    H(q);
    MResetZ(q)
}
```

**[Slide: fallback code]**

If yours looks very different or doesn't run, just copy this. Four lines. That's a quantum program.

### Run It

Click the Run button above `Main()` — or Ctrl+F5 / Cmd+F5.

You should see either `One` or `Zero`. Run it again. You might get a different answer. That's the point.

### See the Histogram

**[Action: run with histogram — show the exact menu/command to do this]**

Now let's run it many times and see the pattern.

You should see roughly 50% Zero, 50% One.

**[Wait for people to get there. "Raise a hand when you see a roughly 50/50 histogram."]**

### Why Is It 50/50?

Here's what just happened. A classical bit is always 0 or 1. A qubit can be put into _superposition_ — a combination of both — until you measure it. The H gate does that. Measurement collapses it to one or the other, with equal probability.

That's the single most important idea in quantum computing. A qubit in superposition isn't secretly 0 or 1 — it genuinely hasn't decided yet. And that property is what makes quantum algorithms possible.

You just demonstrated it.

### See the Circuit

**[Action: show how to view the circuit diagram]**

This is the circuit diagram for your program. You can read it left to right: the qubit starts at |0⟩, passes through the H gate (which creates superposition), and then gets measured.

One qubit, one gate, one measurement. Simplest quantum program there is.

### Ask Copilot to Explain

Now ask Copilot:

> Explain how this code works step by step.

Read what it gives you. You can ask follow-ups. Try: "What does the H gate do mathematically?" or "Why do I need MResetZ instead of just M?" This is Copilot as a tutor — ask it anything.

**[Give people 30-60 seconds to read their explanation. Don't narrate over it.]**

### Break It and Fix It (watch me)

**[Presenter only — audience watches]**

Now I'm going to introduce a bug. I'll delete the H gate.

**[Delete the H(q) line. Run it.]**

Now it always returns Zero. The program "works" but it's not random anymore. It's a boring classical bit.

Watch this. I'll start a new Copilot chat and ask:

> This program is supposed to be a quantum random coin flip, but it always returns Zero. Find and fix the bug.

**[Let Copilot respond. It should identify the missing H gate.]**

It read the code, understood the intent, found the problem, and fixed it. I didn't tell it what was wrong.

### Debug It (watch me)

**[Presenter only — audience watches]**

One more thing. I'll set a breakpoint on the measurement line. Now I'll debug (F5).

**[Step through. Show the qubit state panel — the qubit should show a superposition state before measurement.]**

See this? Before measurement, the qubit is in superposition — the debugger shows you the quantum state. After measurement, it collapses. You can watch the quantum mechanics happening step by step.

---

## 4. Quantum Katas (~5 min)

**[Action: open the QDK Learning panel]**

Everything we just did was freeform exploration. But the QDK also has a structured learning path built right into VS Code — the Quantum Katas.

**[Show the panel. Navigate to the first exercise.]**

These are guided exercises. Each one teaches a single concept, gives you a small coding problem, and checks your answer. You can ask Copilot for hints along the way.

**[Show starting an exercise. Show what the checker looks like when you pass.]**

If you want to keep learning after today, this is where you go. There are exercises on superposition, entanglement, quantum algorithms — the whole progression. And it's all right here in your editor.

---

## 5. Challenge (~10 min)

**[Slide: rotating challenge slides]**

Alright. Your turn. You have 10 minutes. Here are three challenges. Each one you complete earns you a raffle ticket for the grand prize — a laptop.

**Challenge 1: Complete the first Quantum Kata.**
Open the Learning Panel. Complete the first exercise. Show me the green checkmark.

**Challenge 2: Show a Bell state histogram.**
Ask Copilot to help you write a program that entangles two qubits and measures them. You're looking for a histogram that shows roughly 50% |00⟩ and 50% |11⟩ — and nothing else. That pattern means the qubits are entangled: they always agree.

**Challenge 3: Make any circuit diagram.**
Generate a circuit diagram from any Q# program. Could be the coin flip, could be something new. Show me the diagram.

**Mobile option:** Go to https://quantum.microsoft.com and complete the first Quantum Kata in the browser.

#### Contest rules

- **Time limit:** 10 minutes.
- **Tools allowed:** Copilot, Google, friends, anything. This isn't a closed-book exam — it's a race.
- **Earning tickets:** Walk up and show me your screen for each challenge completed. One raffle ticket per challenge.
- **Drawing:** We'll use a quantum random number generator program in the QDK to draw the winner from the ticket pool.
- **Tip to announce:** "You can use Copilot to help you with all of these. I literally just showed you how."

#### Challenge slides

Use three rotating challenge slides. Each should be readable from the back of the room and understandable in under five seconds.

**Slide 1: Complete the first Quantum Kata**

- Success: show the completed first kata.
- Hint: start from the Quantum Katas section in the Activity Bar.

**Slide 2: Show a Bell state histogram**

- Success: run a Bell state example and show the histogram output.
- Hint: look for results close to 50/50 between `00` and `11`.

**Slide 3: Make any circuit diagram**

- Success: display any generated quantum circuit diagram.
- Hint: use a simple circuit if you are new.

---

## 6. Prize Drawing (~2 min)

**[After timer goes off]**

Time's up! Let's draw the winner.

Remember the random number generator we wrote at the beginning? Let's put it to work.

**[Modify the coin flip program to generate a random number in the range 1..N where N is the number of raffle tickets handed out. Or have this prepared in advance with a placeholder for N.]**

**[Run it. Announce the ticket number. Award the laptop.]**

That number was generated by a qubit in superposition — genuinely random, not pseudo-random. The same principle we learned today.

Thanks everyone. Everything you need to keep going is already installed on your machine.

---

## Notes for presenter

**Fallback plan if Copilot is slow/down:** Have the coin flip code on a slide. The demo still works — you just say "here's the code" instead of generating it. The explain/fix/debug parts are presenter-only anyway.

**Fallback plan if debugger is flaky:** Skip it. It's a "watch me" moment and not essential to the flow.

**The Bell state challenge is intentionally not demoed.** They saw the tools. Now they apply them to a new concept (entanglement) on their own. Copilot will guide them. This is where the learning happens.

**Slide deck needed:**

1. Title / intro
2. Installation steps
3. Copilot prompt: "Write a Q# program that flips a quantum coin..."
4. Fallback code (the 4-line program)
5. Challenge 1 (rotating)
6. Challenge 2 (rotating)
7. Challenge 3 (rotating)

**What to prepare in advance:**

- A blank VS Code window with QDK + Copilot ready
- The raffle-drawing program (coin flip extended to N tickets) ready to paste
- The installation handout link sent out beforehand
- Physical raffle tickets (numbered)

**Dry run notes:**

- For setup, use a VM, profile, or other clean environment for the install walkthrough, then switch back to the already-installed environment for the rest of the demo. Do not risk the whole talk on a live install.
- Slides should give orientation; demos should carry the substance.
- The entire thing may be on one small screen — keep transitions clean between slides and live demo.
