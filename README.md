# Ant Colony Simulation

Simulate a chaotic space ant invasion.

---

## Overview

You’re given a map of colonies connected by tunnels (`north`, `south`, `east`, `west`).  
Each line in the map file has the format:

Colony direction=Neighbor [direction=Neighbor ...]

Example:
Fizz north=Buzz west=Bla south=Blub
Buzz south=Fizz west=Blip


Ants spawn randomly in colonies and move randomly along tunnels each iteration.

---

## Rules

- Start with **N ants** (given as a command-line argument).  
- Each turn, ants move randomly to a connected colony.  
- If **two ants meet**, they destroy each other **and** the colony: "Fizz has been destroyed by ant 10 and ant 34!"
- Destroyed colonies and tunnels are removed from the map.  
- The simulation ends when: all ants are dead, or each has moved at least **10,000 times**.

---

## Usage

cargo run -- <num_ants>

Example:
cargo run -- 100

At the end, the remaining world is printed in the same format as the input.

---

## Example

**colony_map.txt**  
Fizz north=Buzz west=Bla  
Buzz south=Fizz west=Blub  
Bla north=Fizz west=Blip east=Blub  
Blip east=Bla south=Blub  
Blub north=Blip west=Bla east=Buzz  

**Output:**  
Buzz has been destroyed by 2, 9!  
Fizz has been destroyed by ant 3, 7!  
Bla west=Blip east=Blub  
Blip east=Bla south=Blub  
Blub north=Blip west=Bla  