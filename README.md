# radarflow
A Web radar for CS:GO using [memflow](https://github.com/memflow/memflow)

## How can I run this?
First, you need to set up a virtual machine on linux using qemu.  
As of now, memflow's pcileech connector is not supported.

How to set up a VM on linux is way out of scope for this. You can find plenty of information online on how to do it.

After you have set up your VM, you can clone this repository on your host:  
`git clone https://github.com/superyu1337/radarflow.git`

Now you can run radarflow:  
`cargo run --release`

For an overview of CLI commands, run this:  
`cargo run --release -- --help`

## Detection Status
VAC: ✅ (Undetected)  
FaceIt: ❓ (Unknown, could work with proper spoofing)  
ESEA: ❓ (Unknown, could work with proper spoofing)  
