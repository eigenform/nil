
## Terminology
Here's a quick overview of some project-specific/domain-specific terms that
I might use when describing things:

- **Frontend/Guest/Source**, refers to the the machine being emulated

- **Backend/Host/Target**, refers to the machine used to emulate another

- **Instruction Set Architecture (ISA)**, an abstract model of a machine
  (i.e. `x86_64`, ARM, PowerPC, etc.)

- **Intermediate Representation (IR)**, an intermediate model of a machine 
  (more generic than a particular ISA) used to represent guest code

- **Lifting**, the act of translating from guest code into the IR

- **Emitting/Recompiling**, the act of translating from the IR into the
  host ISA

- **Register Allocation**, the process of mapping from abstract storage
  locations (in the IR) to physical registers in the target ISA

- **Basic Block**, an atomic unit of execution in the IR, corresponding to
  blocks of straight-line code in the guest ISA, terminated by branching 
  operations

- **Dispatcher**, a function used to call into recompiled code, emitted once
  during runtime

- **Runtime Interface**, a set of reserved registers holding pointers to the
  guest machine's state


## Top-level Organization
nil exposes a `nil::Jit` type to users, which is a container for the top-level 
state associated with the guest machine.

The body of the JIT is a loop is implemented in `nil::Jit::run(&mut self)`, 
which continously lifts basic blocks of not-yet-visited guest code, then 
recompiles and executes them natively on the host.

## Runtime
A `nil::runtime::RuntimeContext` enshrines the conventions used during runtime 
for calling into recompiled code, and the interfaces used by recompiled code
for interacting with aspects of the guest machine's state.

### Runtime Interface
A `RuntimeContext` is initialized with a set of raw pointers which act as
interfaces to the state of the guest machine. When emitting the dispatcher
block, we specify that these pointers are moved into some set of reserved
registers before calling into a recompiled block:

- A pointer to (a contiguous `u32` array of) general-purpose register values,
- A pointer to the current program status register value,
- A pointer to the base of "fast memory" (guest memory that we identity-map 
  into the virtual address space of the process on the host),
- A pointer to a value representing a CPU cycle-counter
- ...

Currently, the runtime interface is defined as:

| Register | Description                                                |
| -------- | ---------------------------------------------------------- |
| `r13`    | Pointer to the current program status register.            |
| `r14`    | Pointer to the base of the "fast memory" area.             |
| `r15`    | Pointer to the base of guest general-purpose registers.    |

### Dispatcher
When a `RuntimeContext` is created, it emits a suitable "dispatcher block,"
which is used for jumping into recompiled basic blocks. The dispatcher block
performs the following:

1. Since we call the dispatcher block from Rust, we first need to respect the 
   SysV ABI by pushing the callee-save registers onto the stack.
2. Move 

Right now, the dispatcher executes one block at a time, and the flow of 
execution looks like this:

	1. Fetch the pointer to the next recompiled block at the program counter
	2. Enter the dispatcher
	3. Execute the next recompiled block (changing the program counter)
	4. Exit the dispatcher
	5. Go back to step #1.

At some point, we probably want blocks to branch directly into other blocks
or something, instead of doing one at a time. Not sure exactly how I plan to
deal with that yet.


