use anyhow::{bail, Context, Error, Result};
use std::path::{Path, PathBuf};
use wizer::Wizer;
use binaryen::{Module, CodegenConfig};

pub(crate) struct Optimizer<'a> {
    optimize: bool,
    script: PathBuf,
    wasm: &'a [u8],
}

impl<'a> Optimizer<'a> {
    pub fn new(wasm: &'a [u8], script: PathBuf) -> Self {
        Self {
            wasm,
            script,
            optimize: false,
        }
    }

    pub fn optimize(self, optimize: bool) -> Self {
        Self { optimize, ..self }
    }

    pub fn write_optimized_wasm(self, dest: impl AsRef<Path>) -> Result<(), Error> {
        let dir = self
            .script
            .parent()
            .filter(|p| p.is_dir())
            .context("input script is not a file")?;

        let mut wasm = Wizer::new()
            .allow_wasi(true)
            .inherit_env(true)
            .dir(dir)
            .run(self.wasm)?;


        if self.optimize {
            if let Ok(mut module) = Module::read(&wasm) {
                module.optimize(&CodegenConfig {
                    shrink_level: 2,
                    optimization_level: 3,
                    debug_info: false
                });
                wasm = module.write();
            } else  {
                bail!("Unable to read wasm binary for wasm-opt optimizations");
            }
        }

        std::fs::write(dest.as_ref(), wasm)?;

        Ok(())
    }
}
