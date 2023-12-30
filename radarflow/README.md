# radarflow
A Web radar for CS2 using [memflow](https://github.com/memflow/memflow)

## How can I run this?
There is two ways to run this, first way is using a KVM/QEMU setup to target a running VM to read memory out of it. The second way is using pcileech hardware, like a PCIe Screamer.

> [!NOTE]  
> The pcileech method is untested. However, I have ordered hardware, and will test soon.

### The KVM/QEMU method
First, you need to set up a virtual machine on linux using qemu.  
How to set up a VM on linux is way out of scope for this. You can find plenty of information online on how to do it.

Before you begin, install the necessary memflow plugins using memflowup from the *stable 0.2.0 channel!*  

Clone the repo on your vm host:  
`git clone https://github.com/superyu1337/radarflow2.git`

Run radarflow:   
`cargo run --release`

For an overview of CLI commands, run this:  
`cargo run --release -- --help`

### The pcileech method

> [!WARNING]  
> The pcileech method is untested.

Install your pcileech hardware in your target pc. On your attacking pc, install the necessary memflow plugins using memflowup from the *stable 0.2.0 channel!*

Clone the repo on your attacking pc:  
`git clone https://github.com/superyu1337/radarflow2.git`

Run radarflow:   
`cargo run --release`

For an overview of CLI commands, run this:  
`cargo run --release -- --help`

## Detection Status
VAC: ✅ (Undetected)  
FaceIt: ❓ (Unknown, could work with proper spoofing on pcileech method)  
ESEA: ❓ (Unknown, could work with proper spoofing on pcileech method)  
