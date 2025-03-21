use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    let args: BuildArgs = BuildArgs {
        elf_name: Some(String::from("obsidian-program")),
        output_directory: Some(String::from("../.artifacts")),
        ..Default::default()
    };
    build_program_with_args("../program", args)
}
