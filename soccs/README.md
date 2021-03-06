<h1 align="center">SOCCs</h1>

<p align="center">
    <a href="https://github.com/Simula-UiB/CryptaPath/blob/master/AUTHORS"><img src="https://img.shields.io/badge/authors-SimulaUIB-orange.svg"></a>
    <a href="https://github.com/Simula-UiB/CryptaPath/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

## About
__SOCCs__ is a way to quickly get started with differential and linear analysis of
SPN-based ciphers using Compressed Right-Hand Side equations (`CRHS equations`). 
The library already has 24 ciphers (including variations of same ciphers) supported, and
also provides suport to analyse your own cipher by the means of the `Cipher` trait. To get 
started with your own cipher, see #Supporting your own cipher.

---

**CREDIT:** The ciphers supported originates from the [CryptaGraph]((https://eprint.iacr.org/2018/764.pdf))
project (Reference [3] in the main README). All code in the folder `cg_original` (short for CryptaGraph original)
are copied from the CryptGraph [repo](https://gitlab.com/psve/cryptagraph/-/find_file/master). Some parts may 
have been modified to fit this project.

---

**WARNING:** This library was developed in an academic context and no part of 
this code should be use in any production system.

---

**WARNING:** This library was made as a prototype and proof-of-concept. As a consequence, the code 
needs a thorough cleanup, and *breaking changes will happen*. However, following the steps for 
*Supporting your own cipher* should minimize the work needed to fix any breaking changes. (Not a 
promise, but a goal).

## Licence
__SOCCs__ is licensed under the MIT License.

* MIT license ([LICENSE](../LICENSE) or http://opensource.org/licenses/MIT)



## How to use
**SOCCS** can be used both as a library and as a command line interface (CLI). Using the CLI is the easiest
and recommended way to use **SOCCS**. There is also possible to analyse your own SPN based cipher by means
of using the *CryptaGraph* defined `Cipher` trait:

### Supporting your own cipher
 The easiest way to use **SOCCS** to analyse your own cipher is to:
1) Implement the `Cipher` trait, as defined in `cg_original\cipher\mod.rs`
2) Add you cipher in the folder `cg_original\cipher\`.
3) Update the function `name_to_cipher` to include you cipher (found in `cg_original\cipher\mod.rs`).
4) Your cipher should now be available as an argument in the CLI.

###  CLI
**SOCCS** CLI have three "*modes*": `linear`, `differential` and `cg`, each with their own set of options.
The two first ones will run a linear or a differential analysis, respectively. The last one will run pre-defined
batches of the ciphers used in the CryptaGraph project. (This allowed us to not having to baby-sit our run-throughs, and
to do them in parallel).

To run either mode and to see the help text explaining each available flag and option, run 

```bash
cargo build -p crush <mode> --help
```
For example, running
```bash
cargo build -p crush differential --help
```
will print info about all the flags and options available for the differential mode.

*Note that there may be different flags and options available to the different modes.*

---
*Some things to note:*
- Any cipher listed in the function `name_to_cipher` (found in soccs\src\dl\cg_original\cipher\mod.rs) is 
 accepted following the `--cipher` option.
- Analysing a cipher is a three-step process: 
  1) First a System of CRHS equations is built from the cipher spec.
  2) The SOC is solved, pruning when the number of nodes exceeds the given threshold.
  3) The solved SOC is then analysed for differential\linear trails\hulls.
- Step 2) may be very time-consuming. It is therefore possible to store the solved SOC to file, for later reuse. 
- Step 3) may load a solved SOC from a file, instead of first performing step 1) and 2).
- The process of solving a SOC is memory consuming. A threshold is therefore needed, for when the pruning process
 should kick in. This is defined as the maximum number of nodes a SOC may contain before pruning. The CryptaPath
 paper talks more about this, except how to set this threshold. Unfortunately, we don't have an automatic way
 to set this threshold yet. Simply put, it should be based on available RAM, where RAM / memory_per_node = threshold.   
 Our current method for setting the threshold is fairly unscientific: Start small, and make a note of the RAM consumption.
 If you still have RAM available, you may consider increasing the threshold.
- *Note that ciphers with larger S-boxes requires a lower threshold than ciphers with smaller S-boxes.*
 This is explained in the paper.

## Plan for the future

- Clean and simplify the logic.
- Apply Clippy and Rustfmt 
- Improve documentation
- Handle log data
- Handle results more cleanly
- ...

## Known issues
TBI
- Implementing a cipher for CryptaPath should ideally also mean that it can be used in SOCCS as is. This is not the
 case at the moment. This is due to the fact that the `cipher` trait as defined by CryptaGraph does not suit the needs
 of CryptaPath.
- Ciphers with a reflective structure similar to PRINCE requires special handling. See in-code notes in cli\main.rs
- ...

## Naming
A collection of `CRHS equations` where all the `CRHS equations` are related is
known as a _System of CRHS equations_, a _SOC_. All the SOCs generated
with this binary are based on cryptographic primitives, and the name thus became
__SOCCs__: __Systems of Crypto-based Compressed Right-Hand Side equations__.
