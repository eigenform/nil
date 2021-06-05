
## Top-level Organization
nil exposes a `nil::Jit` type to users, which is a container for the top-level 
state associated with the guest machine.

The body of the JIT is a loop is implemented in `nil::Jit::run(&mut self)`, 
which continously lifts basic blocks of not-yet-visited guest code, then 
recompiles and executes them natively on the host.

## Emitting Code during Runtime
I'm using the [dynasm](https://github.com/CensoredUsername/dynasm-rs) crate
for assembling/emitting x64. So far, I haven't looked at how/where this crate 
actually allocates generated code *at all*. I'll reorganize things later when 
that becomes clearer (i.e. when I see that bad locality ruins my perf).

## Runtime Behavior
A `nil::runtime::RuntimeContext` enshrines the conventions used for calling 
into our recompiled code, and the interfaces used by recompiled code for 
interacting with aspects of the guest machine's state.

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
2. Fill out reserved registers with runtime context pointers.
3. Call into the recompiled block.
4. Restore the callee-save registers and the stack pointer, then return.

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

## Memories
Ideally, there's some interface that we want users to implement, in order to
deal with loads and stores to memories, i.e. because: 

- You might want to emulate a physical memory map
- You might want to emulate an MMU
- You might need to emulate loads and stores to memory-mapped I/O

### "Fast Memory"
Currently, `nil::mem` only has a simple mechanism for embedding a guest's 
32-bit address space inside the host process' virtual address space
(right now, the base is assumed to be `0x0000_1337_0000_0000`). 
This is sufficient to run and test the library with simple binaries.
This means that during code generation, we don't have to translate addresses,
and we can simply emit loads and stores with the original 32-bit address.

## Other Topics
Other related design problems, notes, and reference material.

### Representing Flags and Conditions
ARM has the notion of status flags (negative, zero, carry, and overflow),
which are stored in a program status register (CPSR). Instruction encodings 
also have a condition field, which means that some instructions are only
executed if a condition is satisfied

On the host (`x86_64`), the FLAGS register looks like this:

| Bit | Mask   | Description |
| --- | ------ | ---------------- |
| 0   | 0x0001 | (CF) Carry flag |
| 1   | 0x0002 | |
| 2   | 0x0004 | (PF) Parity flag |
| 3   | 0x0008 | |
| 4   | 0x0010 | (AF) Adjust flag |
| 5   | 0x0020 | |
| 6   | 0x0040 | (ZF) Zero flag	|
| 7   | 0x0080 | (SF) Sign flag	|


## Terminology
There are a lot of moving parts in this kind of thing. Here's a quick overview 
of a connected set of terms I might've used when describing things:

- **Basic Block**: an atomic unit of execution in the IR, corresponding to
  blocks of straight-line code in the guest ISA, terminated by branching 
  operations.

- **Code Generation/Emitting/Lowering**: the act of translating from the IR 
  into the target ISA.

- **Dispatcher**: function used to call into recompiled code, emitted once
  during runtime.

- **Guest**: the machine being emulated.
- **Host/Target**: the machine running the emulator.

- **Instruction Set Architecture (ISA)**: an abstract model of a machine
  (i.e. `x86_64`, ARM, PowerPC, etc).

- **Intermediate Representation (IR)**: an intermediate model of a machine 
  (more generic than a particular ISA) used to represent guest code.

- **Lifting**: the act of translating from guest code into the IR.
- **Local Optimization**: optimizations within individual basic blocks.

- **Recompiling**: the whole process of lifting + optimizing + emitting.

- **Register Allocation**: the process of mapping from abstract storage
  locations (in the IR) to physical registers in the target ISA.

- **Runtime Interface**: a set of reserved registers holding pointers to the
  guest machine's state.

- **Spilling/Spilled Values**: when generating code, values that cannot be
  allocated in physical registers are "spilled" to some other kind of available
  storage (i.e. memory, typically the stack).

- **Static single assignment form (SSA)**: a property of an IR whose model
  has a [static, single] storage location for each unique value/variable.


